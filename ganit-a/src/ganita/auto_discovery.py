"""
Continuous auto-discovery mode.

Runs indefinitely, exploring parameter space and resuming from checkpoints.
"""

from __future__ import annotations

import logging
import random
import signal
import time
from pathlib import Path
from typing import Any

from ganita.checkpoint import ExplorationCheckpoint
from ganita.models import Discovery, ModuleConfig, RunConfig
from ganita.modules.continued_fractions import ContinuedFractionsModule
from ganita.modules.fixed_points import FixedPointsModule
from ganita.modules.series_products import SeriesProductsModule
from ganita.orchestrator import Orchestrator
from ganita.services.evaluator import Evaluator
from ganita.services.recognizer import Recognizer
from ganita.storage.db import Database

logger = logging.getLogger(__name__)


class AutoDiscovery:
    """Continuous auto-discovery system with checkpointing."""

    def __init__(
        self,
        config: RunConfig,
        db_path: Path | str,
        artifacts_path: Path | str,
        checkpoint_path: Path | str,
    ):
        """
        Initialize auto-discovery.

        Args:
            config: Base configuration
            db_path: Database path
            artifacts_path: Artifacts directory
            checkpoint_path: Checkpoint file path
        """
        self.config = config
        self.db_path = Path(db_path)
        self.artifacts_path = Path(artifacts_path)
        self.checkpoint = ExplorationCheckpoint(checkpoint_path)

        self.running = False
        self.paused = False

        # Signal handling for graceful shutdown
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)

        # Initialize services
        self.db = Database(db_path)
        self.evaluator = Evaluator(config.evaluator)
        self.recognizer = Recognizer(config.recognizer)

        # Statistics
        self.stats = {
            "session_discoveries": 0,
            "session_iterations": 0,
            "session_start": None,
            "last_discovery_time": None,
        }

    def _signal_handler(self, signum: int, frame: Any) -> None:
        """Handle shutdown signals gracefully."""
        logger.info(f"Received signal {signum}, shutting down gracefully...")
        self.stop()

    def start(self, continuous: bool = True, max_iterations: int | None = None) -> None:
        """
        Start auto-discovery.

        Args:
            continuous: If True, runs indefinitely until stopped
            max_iterations: Maximum iterations (None = unlimited)
        """
        self.running = True
        self.stats["session_start"] = time.time()

        logger.info("=" * 70)
        logger.info("AUTO-DISCOVERY MODE STARTED")
        logger.info("=" * 70)
        logger.info(f"Checkpoint: {self.checkpoint.checkpoint_path}")
        logger.info(f"Database: {self.db_path}")
        logger.info(f"Mode: {'Continuous' if continuous else 'Limited'}")
        if max_iterations:
            logger.info(f"Max iterations: {max_iterations}")

        # Show progress from checkpoint
        progress = self.checkpoint.get_progress()
        logger.info(f"\nResuming from checkpoint:")
        logger.info(f"  Total discoveries so far: {progress['total_discoveries']}")
        logger.info(f"  Total iterations: {progress['total_iterations']}")

        self.checkpoint.set_status("running")

        iteration = 0

        try:
            while self.running:
                if self.paused:
                    time.sleep(1)
                    continue

                iteration += 1

                # Check max iterations
                if max_iterations and iteration > max_iterations:
                    logger.info(f"Reached max iterations ({max_iterations})")
                    break

                logger.info(f"\n{'='*70}")
                logger.info(f"ITERATION {iteration}")
                logger.info(f"{'='*70}")

                # Run discovery cycle
                self._discovery_cycle(iteration)

                # Show progress
                self._show_progress()

                # Brief pause between cycles
                if continuous and self.running:
                    time.sleep(2)

        except Exception as e:
            logger.error(f"Error in auto-discovery: {e}", exc_info=True)
            self.checkpoint.set_status("error")

        finally:
            self._cleanup()

    def _discovery_cycle(self, iteration: int) -> None:
        """Run one discovery cycle."""

        # Explore each enabled module
        for module_config in self.config.modules:
            if not module_config.enabled:
                continue

            if not self.running or self.paused:
                break

            module_name = module_config.name
            logger.info(f"\n--- Module: {module_name} ---")

            # Get module state
            module_state = self.checkpoint.get_module_state(module_name)

            # Generate new parameters
            params = self._generate_next_params(module_name, module_config)

            if params is None:
                logger.info(f"No new parameters for {module_name}, generating random...")
                params = self._generate_random_params(module_config)

            # Check if already explored
            if self.checkpoint.is_param_explored(module_name, params):
                logger.debug(f"Parameters already explored, generating new ones...")
                params = self._generate_random_params(module_config)

            logger.info(f"Exploring parameters: {params}")

            # Explore with these parameters
            discoveries = self._explore_module(module_name, params)

            # Update checkpoint
            self.checkpoint.update_module_state(
                module_name,
                params=params,
                discoveries_count=module_state["discoveries_count"] + len(discoveries),
            )

            # Store discoveries
            if discoveries:
                self._store_discoveries(discoveries, iteration)
                self.stats["session_discoveries"] += len(discoveries)
                self.stats["last_discovery_time"] = time.time()

            self.stats["session_iterations"] += 1

    def _generate_next_params(
        self, module_name: str, module_config: ModuleConfig
    ) -> dict[str, Any] | None:
        """Generate next unexplored parameter combination."""
        # Get parameter grid
        grid_spec = module_config.parameter_grid

        if not grid_spec:
            return None

        # For auto-discovery, we intelligently expand the search space
        # beyond the initial grid by varying parameters randomly

        state = self.checkpoint.get_module_state(module_name)

        # If we've explored less than 10 params, use grid
        # Otherwise, generate random variations
        if len(state["explored_params"]) < 10:
            # Use standard grid generation (simplified)
            return None

        # Generate variations of successful parameters
        return self._generate_random_params(module_config)

    def _generate_random_params(self, module_config: ModuleConfig) -> dict[str, Any]:
        """Generate random parameter combination."""
        grid_spec = module_config.parameter_grid

        if not grid_spec:
            return {}

        params = {}

        for key, value in grid_spec.items():
            if isinstance(value, list):
                params[key] = random.choice(value)
            elif isinstance(value, dict):
                if value.get("type") == "range":
                    start = value["start"]
                    end = value["end"]
                    step = value.get("step", 1)
                    choices = list(range(start, end, step))
                    params[key] = random.choice(choices) if choices else start
                elif value.get("type") == "linspace":
                    # Random value in range
                    start = value["start"]
                    end = value["end"]
                    params[key] = random.uniform(start, end)
            else:
                params[key] = value

        return params

    def _explore_module(self, module_name: str, params: dict[str, Any]) -> list[Discovery]:
        """Explore with a specific module and parameters."""
        # Create module instance with single parameter combination
        env_snapshot = {"auto_discovery": True, "iteration": self.stats["session_iterations"]}

        module_config = {
            "parameter_grid": {},  # Empty grid, we'll explore one set
            "enabled": True,
        }

        discoveries = []

        try:
            if module_name == "series_products":
                module = SeriesProductsModule(
                    self.evaluator, self.recognizer, module_config, env_snapshot
                )
                # Explore single parameter set
                disc = module._explore_by_template(params)
                if disc:
                    discoveries.append(disc)

            elif module_name == "continued_fractions":
                module = ContinuedFractionsModule(
                    self.evaluator, self.recognizer, module_config, env_snapshot
                )
                disc = module._explore_by_template(params)
                if disc:
                    discoveries.append(disc)

            elif module_name == "fixed_points":
                module = FixedPointsModule(
                    self.evaluator, self.recognizer, module_config, env_snapshot
                )
                disc = module._explore_by_template(params)
                if disc:
                    discoveries.append(disc)

        except Exception as e:
            logger.error(f"Error exploring {module_name} with params {params}: {e}")

        return discoveries

    def _store_discoveries(self, discoveries: list[Discovery], iteration: int) -> None:
        """Store discoveries in database."""
        # Create a mini-run for this iteration
        from ganita import __version__

        run_id = self.db.create_run(
            orchestrator_ver=__version__,
            code_hash=f"auto-discovery-{iteration}",
            config={"auto_discovery": True, "iteration": iteration},
        )

        self.checkpoint.add_run(run_id)

        for disc in discoveries:
            disc_id = self.db.insert_discovery(run_id, disc)
            logger.info(f"  ✓ Stored discovery #{disc_id}: {disc.value_repr[:50]}...")

            # Store trace
            if disc.evidence.interval_trace:
                from ganita.storage.artifacts import ArtifactStore

                artifacts = ArtifactStore(self.artifacts_path)
                artifacts.write_trace(disc_id, disc.evidence.interval_trace)

            # Store recognition
            if disc.evidence.recognizer_summary:
                rec = disc.evidence.recognizer_summary
                self.db.add_recognition(
                    disc_id,
                    rec.basis_name,
                    rec.precision_used,
                    rec.success,
                    rec.relation_tex,
                    rec.relation_json,
                    rec.height,
                    rec.residual_log10,
                )

        self.db.finish_run(run_id)

    def _show_progress(self) -> None:
        """Show current progress."""
        progress = self.checkpoint.get_progress()
        elapsed = time.time() - self.stats["session_start"]

        logger.info(f"\n{'='*70}")
        logger.info("PROGRESS UPDATE")
        logger.info(f"{'='*70}")
        logger.info(f"Session discoveries: {self.stats['session_discoveries']}")
        logger.info(f"Session iterations: {self.stats['session_iterations']}")
        logger.info(f"Total discoveries: {progress['total_discoveries']}")
        logger.info(f"Total iterations: {progress['total_iterations']}")
        logger.info(f"Elapsed time: {elapsed:.1f}s")

        if self.stats["session_discoveries"] > 0:
            rate = self.stats["session_discoveries"] / elapsed * 60
            logger.info(f"Discovery rate: {rate:.2f} per minute")

        logger.info(f"\nModule Status:")
        for name, mod_progress in progress["modules"].items():
            logger.info(
                f"  {name}: {mod_progress['discoveries']} discoveries, "
                f"{mod_progress['explored_count']} params explored"
            )

    def pause(self) -> None:
        """Pause discovery (can be resumed)."""
        logger.info("Pausing auto-discovery...")
        self.paused = True
        self.checkpoint.set_status("paused")

    def resume(self) -> None:
        """Resume discovery."""
        logger.info("Resuming auto-discovery...")
        self.paused = False
        self.checkpoint.set_status("running")

    def stop(self) -> None:
        """Stop discovery gracefully."""
        logger.info("Stopping auto-discovery...")
        self.running = False

    def _cleanup(self) -> None:
        """Cleanup and finalize."""
        logger.info("\n" + "=" * 70)
        logger.info("AUTO-DISCOVERY STOPPED")
        logger.info("=" * 70)

        self._show_progress()

        self.checkpoint.set_status("stopped")

        logger.info(f"\nCheckpoint saved to: {self.checkpoint.checkpoint_path}")
        logger.info("To resume, run the same command again.")

        # Export summary
        summary = self.checkpoint.export_summary()
        summary_path = self.checkpoint.checkpoint_path.with_suffix(".summary.txt")
        with open(summary_path, "w") as f:
            f.write(summary)
        logger.info(f"Summary exported to: {summary_path}")


# Helper method additions to modules
def _explore_by_template(self, params: dict[str, Any]) -> Discovery | None:
    """Explore a single parameter set. Add to each module class."""
    template = params.get("template", "")

    # Route to appropriate exploration method
    if hasattr(self, f"_explore_{template}"):
        method = getattr(self, f"_explore_{template}")
        return method(params)

    return None


# Monkey-patch modules with the helper
SeriesProductsModule._explore_by_template = _explore_by_template
ContinuedFractionsModule._explore_by_template = _explore_by_template
FixedPointsModule._explore_by_template = _explore_by_template
