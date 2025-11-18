"""
Base module interface for discovery modules.

All discovery modules inherit from BaseModule.
"""

from __future__ import annotations

import logging
from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from ganita.models import Discovery
    from ganita.services.evaluator import Evaluator
    from ganita.services.recognizer import Recognizer

logger = logging.getLogger(__name__)


class BaseModule(ABC):
    """Base class for all discovery modules."""

    name: str = "base"
    description: str = "Base module"

    def __init__(
        self,
        evaluator: Evaluator,
        recognizer: Recognizer,
        config: dict[str, Any],
        env_snapshot: dict[str, Any],
    ):
        """
        Initialize module with services.

        Args:
            evaluator: High-precision evaluator
            recognizer: PSLQ/LLL recognizer
            config: Module-specific configuration
            env_snapshot: Environment snapshot for reproducibility
        """
        self.evaluator = evaluator
        self.recognizer = recognizer
        self.config = config
        self.env_snapshot = env_snapshot

    @abstractmethod
    def explore(self) -> list[Discovery]:
        """
        Run exploration and return list of discoveries.

        This is the main entry point for each module.
        """
        raise NotImplementedError

    def _create_parameter_grid(self, spec: dict[str, Any]) -> list[dict[str, Any]]:
        """
        Generate parameter grid from specification.

        Example spec:
        {
            "p": [2, 3, 4],
            "x": {"type": "linspace", "start": 0.1, "end": 0.9, "num": 5}
        }
        """
        import itertools

        # Expand ranges
        expanded = {}
        for key, value in spec.items():
            if isinstance(value, list):
                expanded[key] = value
            elif isinstance(value, dict):
                if value.get("type") == "linspace":
                    import numpy as np

                    expanded[key] = list(
                        np.linspace(
                            value["start"],
                            value["end"],
                            value.get("num", 10),
                        )
                    )
                elif value.get("type") == "range":
                    expanded[key] = list(
                        range(
                            value["start"],
                            value["end"],
                            value.get("step", 1),
                        )
                    )
            else:
                expanded[key] = [value]

        # Generate cartesian product
        keys = list(expanded.keys())
        values_lists = [expanded[k] for k in keys]

        grid = []
        for values in itertools.product(*values_lists):
            grid.append(dict(zip(keys, values)))

        return grid

    def _should_skip_params(self, params: dict[str, Any]) -> bool:
        """
        Check if parameter combination should be skipped.

        Override in subclasses for custom filtering logic.
        """
        return False

    def _log_progress(self, message: str, level: str = "info") -> None:
        """Log progress message."""
        log_func = getattr(logger, level, logger.info)
        log_func(f"[{self.name}] {message}")
