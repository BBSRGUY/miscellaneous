"""
Orchestrator for managing the discovery pipeline.

Coordinates modules, services, and persistence.
"""

from __future__ import annotations

import hashlib
import logging
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
from pathlib import Path
from typing import Any

from ganita import __version__
from ganita.models import Discovery, EnvironmentSnapshot, RunConfig
from ganita.modules.base import BaseModule
from ganita.modules.continued_fractions import ContinuedFractionsModule
from ganita.modules.fixed_points import FixedPointsModule
from ganita.modules.series_products import SeriesProductsModule
from ganita.services.crossval import CrossValidator
from ganita.services.evaluator import Evaluator
from ganita.services.recognizer import Recognizer
from ganita.storage.artifacts import ArtifactStore
from ganita.storage.db import Database

logger = logging.getLogger(__name__)


class Orchestrator:
    """Main orchestrator for mathematical discovery."""

    def __init__(self, config: RunConfig, db_path: str | Path, artifacts_path: str | Path):
        """
        Initialize orchestrator.

        Args:
            config: Run configuration
            db_path: Path to SQLite database
            artifacts_path: Path to artifacts directory
        """
        self.config = config
        self.db = Database(db_path)
        self.artifacts = ArtifactStore(artifacts_path)

        # Initialize services
        self.evaluator = Evaluator(config.evaluator)
        self.recognizer = Recognizer(config.recognizer)

        # Initialize modules
        self.modules = self._init_modules()

        # Cross-validator
        self.cross_validator = CrossValidator(self.modules)

        # Tracking
        self.run_id: int | None = None
        self.start_time: float = 0.0

    def _init_modules(self) -> dict[str, BaseModule]:
        """Initialize all discovery modules."""
        modules = {}

        # Get environment snapshot
        env_snapshot = self._get_env_snapshot()

        # Series and products
        series_config = self._get_module_config("series_products")
        if series_config.get("enabled", True):
            modules["series_products"] = SeriesProductsModule(
                self.evaluator,
                self.recognizer,
                series_config,
                env_snapshot,
            )

        # Continued fractions
        cf_config = self._get_module_config("continued_fractions")
        if cf_config.get("enabled", True):
            modules["continued_fractions"] = ContinuedFractionsModule(
                self.evaluator,
                self.recognizer,
                cf_config,
                env_snapshot,
            )

        # Fixed points
        fp_config = self._get_module_config("fixed_points")
        if fp_config.get("enabled", True):
            modules["fixed_points"] = FixedPointsModule(
                self.evaluator,
                self.recognizer,
                fp_config,
                env_snapshot,
            )

        return modules

    def _get_module_config(self, module_name: str) -> dict[str, Any]:
        """Get configuration for a specific module."""
        for mod_cfg in self.config.modules:
            if mod_cfg.name == module_name:
                return {
                    "enabled": mod_cfg.enabled,
                    "budget_minutes": mod_cfg.budget_minutes,
                    "max_terms": mod_cfg.max_terms,
                    "parameter_grid": mod_cfg.parameter_grid,
                }
        return {"enabled": True}

    def _get_env_snapshot(self) -> dict[str, Any]:
        """Get environment snapshot for reproducibility."""
        code_hash = self._compute_code_hash()
        env = EnvironmentSnapshot.create(code_hash, self.config.random_seed)
        return env.model_dump()

    def _compute_code_hash(self) -> str:
        """Compute hash of source code for reproducibility."""
        # In production, this would hash all Python files
        # For now, use a simple version-based hash
        content = f"{__version__}:{datetime.utcnow().date()}"
        return hashlib.sha256(content.encode()).hexdigest()[:16]

    def run(self) -> int:
        """
        Run the full discovery pipeline.

        Returns run ID.
        """
        self.start_time = time.time()

        # Create run in database
        self.run_id = self.db.create_run(
            orchestrator_ver=__version__,
            code_hash=self._compute_code_hash(),
            config=self.config.model_dump(),
        )

        logger.info(f"Starting run {self.run_id}")

        try:
            # Run modules in parallel
            all_discoveries = self._run_modules_parallel()

            logger.info(f"Generated {len(all_discoveries)} raw discoveries")

            # Store discoveries
            for disc in all_discoveries:
                disc_id = self.db.insert_discovery(self.run_id, disc)

                # Store trace artifact
                if disc.evidence.interval_trace:
                    trace_path = self.artifacts.write_trace(disc_id, disc.evidence.interval_trace)
                    logger.debug(f"Stored trace for discovery {disc_id} at {trace_path}")

                # Store recognition attempt if any
                if disc.evidence.recognizer_summary:
                    rec_sum = disc.evidence.recognizer_summary
                    self.db.add_recognition(
                        disc_id,
                        rec_sum.basis_name,
                        rec_sum.precision_used,
                        rec_sum.success,
                        rec_sum.relation_tex,
                        rec_sum.relation_json,
                        rec_sum.height,
                        rec_sum.residual_log10,
                    )

            # Cross-validation
            logger.info("Running cross-validation...")
            validated_discoveries = self.cross_validator.cross_validate_all(all_discoveries)

            # Update validated discoveries
            for disc in validated_discoveries:
                if disc.stage.value == "cross_validated" and disc.discovery_id:
                    self.db.update_discovery_stage(disc.discovery_id, disc.stage.value)

                    # Record cross-validation
                    for other_module in disc.evidence.cross_validation_methods:
                        self.db.add_cross_validation(
                            disc.discovery_id,
                            other_module,
                            "1e-100",
                            True,
                            {"method": "interval_overlap"},
                        )

            # Find duplicates
            duplicates = self.cross_validator.find_duplicates(all_discoveries)
            if duplicates:
                logger.info(f"Found {len(duplicates)} groups of duplicate values")
                for canonical, disc_ids in duplicates.items():
                    logger.info(f"  Value {canonical[:20]}... found in {len(disc_ids)} discoveries")

            # Finish run
            self.db.finish_run(self.run_id)

            elapsed = time.time() - self.start_time
            logger.info(f"Run {self.run_id} completed in {elapsed:.2f} seconds")

            # Print summary
            stats = self.db.get_run_stats(self.run_id)
            logger.info("Run statistics:")
            logger.info(f"  Total discoveries: {stats.get('total_discoveries', 0)}")
            logger.info(f"  Certified: {stats.get('certified', 0)}")
            logger.info(f"  Recognized: {stats.get('recognized', 0)}")
            logger.info(f"  Cross-validated: {stats.get('cross_validated', 0)}")
            logger.info(f"  Average score: {stats.get('avg_score', 0):.4f}")

            return self.run_id

        except Exception as e:
            logger.error(f"Error in run {self.run_id}: {e}", exc_info=True)
            if self.run_id:
                self.db.finish_run(self.run_id)
            raise

    def _run_modules_parallel(self) -> list[Discovery]:
        """Run all enabled modules in parallel."""
        all_discoveries: list[Discovery] = []

        with ThreadPoolExecutor(max_workers=self.config.max_workers) as executor:
            futures = {}

            for module_name, module in self.modules.items():
                future = executor.submit(self._run_module_with_timeout, module)
                futures[future] = module_name

            for future in as_completed(futures):
                module_name = futures[future]
                try:
                    discoveries = future.result()
                    logger.info(f"Module {module_name} completed with {len(discoveries)} discoveries")
                    all_discoveries.extend(discoveries)
                except Exception as e:
                    logger.error(f"Module {module_name} failed: {e}", exc_info=True)

        return all_discoveries

    def _run_module_with_timeout(self, module: BaseModule) -> list[Discovery]:
        """Run a module with timeout handling."""
        logger.info(f"Starting module {module.name}")
        start_time = time.time()

        try:
            discoveries = module.explore()
            elapsed = time.time() - start_time
            logger.info(f"Module {module.name} completed in {elapsed:.2f}s")
            return discoveries

        except Exception as e:
            logger.error(f"Error in module {module.name}: {e}", exc_info=True)
            return []

    def get_top_discoveries(self, limit: int = 20, stage: str | None = None) -> list[dict[str, Any]]:
        """Get top discoveries from the latest run."""
        if not self.run_id:
            return []

        return self.db.query_top(stage=stage, limit=limit, run_id=self.run_id)


def run_exploration(config: RunConfig, db_path: str | Path, artifacts_path: str | Path) -> int:
    """
    Run a complete exploration session.

    Args:
        config: Run configuration
        db_path: Path to database
        artifacts_path: Path to artifacts directory

    Returns:
        run_id
    """
    orchestrator = Orchestrator(config, db_path, artifacts_path)
    return orchestrator.run()
