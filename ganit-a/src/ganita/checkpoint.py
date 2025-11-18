"""
Checkpoint system for resumable exploration.

Tracks exploration state and allows resuming from where left off.
"""

from __future__ import annotations

import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)


class ExplorationCheckpoint:
    """Manages checkpoints for resumable exploration."""

    def __init__(self, checkpoint_path: Path | str):
        """
        Initialize checkpoint manager.

        Args:
            checkpoint_path: Path to checkpoint file
        """
        self.checkpoint_path = Path(checkpoint_path)
        self.checkpoint_path.parent.mkdir(parents=True, exist_ok=True)

        # Checkpoint data structure
        self.data = {
            "version": "1.0.0",
            "created_at": datetime.utcnow().isoformat(),
            "updated_at": datetime.utcnow().isoformat(),
            "total_discoveries": 0,
            "total_iterations": 0,
            "modules": {},  # module_name -> module_state
            "runs": [],  # List of run IDs
            "status": "initialized",  # initialized, running, paused, completed
        }

        # Load existing checkpoint if it exists
        if self.checkpoint_path.exists():
            self.load()

    def load(self) -> None:
        """Load checkpoint from disk."""
        try:
            with open(self.checkpoint_path, "r") as f:
                self.data = json.load(f)
            logger.info(f"Loaded checkpoint from {self.checkpoint_path}")
        except Exception as e:
            logger.error(f"Error loading checkpoint: {e}")
            # Keep default data

    def save(self) -> None:
        """Save checkpoint to disk."""
        try:
            self.data["updated_at"] = datetime.utcnow().isoformat()

            # Write atomically
            temp_path = self.checkpoint_path.with_suffix(".tmp")
            with open(temp_path, "w") as f:
                json.dump(self.data, f, indent=2)

            temp_path.replace(self.checkpoint_path)
            logger.debug(f"Saved checkpoint to {self.checkpoint_path}")

        except Exception as e:
            logger.error(f"Error saving checkpoint: {e}")

    def get_module_state(self, module_name: str) -> dict[str, Any]:
        """Get state for a specific module."""
        return self.data["modules"].get(module_name, {
            "explored_params": [],  # List of parameter combinations already tried
            "last_params": None,
            "discoveries_count": 0,
            "iterations": 0,
            "status": "pending",
        })

    def update_module_state(
        self,
        module_name: str,
        params: dict[str, Any] | None = None,
        discoveries_count: int | None = None,
        status: str | None = None,
    ) -> None:
        """Update state for a module."""
        if module_name not in self.data["modules"]:
            self.data["modules"][module_name] = self.get_module_state(module_name)

        state = self.data["modules"][module_name]

        if params is not None:
            # Add to explored params
            params_key = json.dumps(params, sort_keys=True)
            if params_key not in state["explored_params"]:
                state["explored_params"].append(params_key)
            state["last_params"] = params

        if discoveries_count is not None:
            state["discoveries_count"] = discoveries_count
            self.data["total_discoveries"] = sum(
                m["discoveries_count"] for m in self.data["modules"].values()
            )

        if status is not None:
            state["status"] = status

        state["iterations"] = state.get("iterations", 0) + 1
        self.data["total_iterations"] += 1

        self.save()

    def is_param_explored(self, module_name: str, params: dict[str, Any]) -> bool:
        """Check if parameter combination has been explored."""
        state = self.get_module_state(module_name)
        params_key = json.dumps(params, sort_keys=True)
        return params_key in state["explored_params"]

    def add_run(self, run_id: int) -> None:
        """Record a run ID."""
        if run_id not in self.data["runs"]:
            self.data["runs"].append(run_id)
        self.save()

    def set_status(self, status: str) -> None:
        """Set overall checkpoint status."""
        self.data["status"] = status
        self.save()

    def get_progress(self) -> dict[str, Any]:
        """Get progress summary."""
        return {
            "total_discoveries": self.data["total_discoveries"],
            "total_iterations": self.data["total_iterations"],
            "modules": {
                name: {
                    "discoveries": state["discoveries_count"],
                    "iterations": state["iterations"],
                    "explored_count": len(state["explored_params"]),
                    "status": state["status"],
                }
                for name, state in self.data["modules"].items()
            },
            "status": self.data["status"],
            "last_updated": self.data["updated_at"],
        }

    def reset(self) -> None:
        """Reset checkpoint (start fresh)."""
        self.data = {
            "version": "1.0.0",
            "created_at": datetime.utcnow().isoformat(),
            "updated_at": datetime.utcnow().isoformat(),
            "total_discoveries": 0,
            "total_iterations": 0,
            "modules": {},
            "runs": [],
            "status": "initialized",
        }
        self.save()

    def export_summary(self) -> str:
        """Export human-readable summary."""
        summary = []
        summary.append("=" * 60)
        summary.append("EXPLORATION CHECKPOINT SUMMARY")
        summary.append("=" * 60)
        summary.append(f"Status: {self.data['status']}")
        summary.append(f"Created: {self.data['created_at']}")
        summary.append(f"Last Updated: {self.data['updated_at']}")
        summary.append(f"Total Discoveries: {self.data['total_discoveries']}")
        summary.append(f"Total Iterations: {self.data['total_iterations']}")
        summary.append(f"Runs: {len(self.data['runs'])}")
        summary.append("\nModule Progress:")
        summary.append("-" * 60)

        for name, state in self.data["modules"].items():
            summary.append(f"\n{name}:")
            summary.append(f"  Discoveries: {state['discoveries_count']}")
            summary.append(f"  Iterations: {state['iterations']}")
            summary.append(f"  Explored Parameters: {len(state['explored_params'])}")
            summary.append(f"  Status: {state['status']}")

        summary.append("=" * 60)

        return "\n".join(summary)
