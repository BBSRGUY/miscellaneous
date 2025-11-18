"""
Cross-validation service for verifying discoveries across multiple methods.
"""

from __future__ import annotations

import logging
from typing import Any

import mpmath as mp

from ganita.models import ConfidenceStage, Discovery
from ganita.services.evaluator import BallInterval

logger = logging.getLogger(__name__)


class CrossValidator:
    """Cross-validates discoveries using independent methods."""

    def __init__(self, modules: dict[str, Any], tolerance: str = "1e-100"):
        """
        Initialize cross-validator.

        Args:
            modules: Dictionary mapping module names to module instances
            tolerance: Tolerance for considering values equal
        """
        self.modules = modules
        self.tolerance = mp.mpf(tolerance)

    def validate(self, discovery: Discovery) -> dict[str, Any]:
        """
        Attempt to reproduce a discovery using other methods.

        Returns validation results with details of successful reproductions.
        """
        results = {
            "validated": False,
            "matching_modules": [],
            "mismatches": [],
            "errors": [],
        }

        # Get value to validate
        target_value = mp.mpf(discovery.value_repr)
        target_interval = BallInterval(
            target_value,
            mp.mpf(discovery.evidence.interval_trace[-1].radius)
            if discovery.evidence.interval_trace
            else mp.mpf("1e-50"),
        )

        # Try other modules
        for module_name, module in self.modules.items():
            # Skip the module that created this discovery
            if module_name == discovery.method:
                continue

            try:
                # Attempt to reproduce with similar parameters
                reproduced = self._try_reproduce(module, discovery.params, target_value)

                if reproduced:
                    repro_value = mp.mpf(reproduced.value_repr)
                    repro_interval = BallInterval(
                        repro_value,
                        mp.mpf(reproduced.evidence.interval_trace[-1].radius)
                        if reproduced.evidence.interval_trace
                        else mp.mpf("1e-50"),
                    )

                    # Check if intervals overlap
                    if target_interval.overlaps(repro_interval):
                        # Check if values are close enough
                        diff = abs(target_value - repro_value)
                        if diff < self.tolerance:
                            results["matching_modules"].append(
                                {
                                    "module": module_name,
                                    "value": str(repro_value),
                                    "difference": str(diff),
                                }
                            )
                            results["validated"] = True
                        else:
                            results["mismatches"].append(
                                {
                                    "module": module_name,
                                    "value": str(repro_value),
                                    "difference": str(diff),
                                }
                            )

            except Exception as e:
                results["errors"].append({"module": module_name, "error": str(e)})
                logger.debug(f"Error validating with {module_name}: {e}")

        return results

    def _try_reproduce(
        self, module: Any, original_params: dict[str, Any], target_value: mp.mpf
    ) -> Discovery | None:
        """
        Try to reproduce a discovery using a different module.

        This is a simplified version - in practice, you'd need more sophisticated
        parameter mapping between different module types.
        """
        # This is a placeholder for demonstration
        # In a full implementation, you'd:
        # 1. Map parameters from one module type to another
        # 2. Run the module's exploration with adapted parameters
        # 3. Check if any discovery matches the target

        # For now, we'll skip actual reproduction
        # This would require running module.explore() which could be expensive

        return None

    def cross_validate_all(self, discoveries: list[Discovery]) -> list[Discovery]:
        """
        Cross-validate all discoveries and promote those that pass.

        Returns updated list of discoveries with validation results.
        """
        validated_discoveries = []

        for disc in discoveries:
            validation_result = self.validate(disc)

            if validation_result["validated"]:
                # Promote to cross-validated stage
                disc.stage = ConfidenceStage.CROSS_VALIDATED
                disc.evidence.cross_validation_methods = [
                    m["module"] for m in validation_result["matching_modules"]
                ]

                # Boost cross-validation score
                disc.rank_features.cross_validation_score = min(
                    1.0, len(validation_result["matching_modules"]) * 0.5
                )

                logger.info(
                    f"Discovery {disc.discovery_id} cross-validated by "
                    f"{len(validation_result['matching_modules'])} modules"
                )

            validated_discoveries.append(disc)

        return validated_discoveries

    def find_duplicates(self, discoveries: list[Discovery]) -> dict[str, list[int]]:
        """
        Find duplicate discoveries (same value from different modules/params).

        Returns mapping of canonical value string to list of discovery IDs.
        """
        value_groups: dict[str, list[int]] = {}

        for disc in discoveries:
            try:
                value = mp.mpf(disc.value_repr)

                # Find if this value matches any existing group
                found_group = False
                for canonical, group in value_groups.items():
                    canonical_val = mp.mpf(canonical)
                    if abs(value - canonical_val) < self.tolerance:
                        if disc.discovery_id is not None:
                            group.append(disc.discovery_id)
                        found_group = True
                        break

                if not found_group:
                    # Create new group
                    canonical_str = str(value)
                    value_groups[canonical_str] = (
                        [disc.discovery_id] if disc.discovery_id else []
                    )

            except Exception as e:
                logger.debug(f"Error processing discovery for duplicates: {e}")

        # Filter to only groups with multiple discoveries
        duplicates = {k: v for k, v in value_groups.items() if len(v) > 1}

        return duplicates
