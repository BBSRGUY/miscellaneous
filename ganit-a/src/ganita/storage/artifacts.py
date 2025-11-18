"""
Artifact storage for interval traces and large blobs.

Stores JSONL files on disk to keep the SQLite database small.
"""

import gzip
import json
from pathlib import Path
from typing import Any

from ganita.models import IntervalSample


class ArtifactStore:
    """Manages on-disk artifact storage."""

    def __init__(self, base_path: Path | str):
        self.base_path = Path(base_path)
        self.base_path.mkdir(parents=True, exist_ok=True)

    def write_trace(
        self, discovery_id: int, trace: list[IntervalSample], compress: bool = True
    ) -> str:
        """Write interval trace to disk and return path."""
        filename = f"trace_{discovery_id}.jsonl"
        if compress:
            filename += ".gz"

        filepath = self.base_path / filename

        if compress:
            with gzip.open(filepath, "wt", encoding="utf-8") as f:
                for sample in trace:
                    json.dump(sample.model_dump(), f)
                    f.write("\n")
        else:
            with open(filepath, "w", encoding="utf-8") as f:
                for sample in trace:
                    json.dump(sample.model_dump(), f)
                    f.write("\n")

        return str(filepath.relative_to(self.base_path.parent))

    def read_trace(self, trace_path: str) -> list[IntervalSample]:
        """Read interval trace from disk."""
        filepath = self.base_path.parent / trace_path
        samples = []

        if filepath.suffix == ".gz":
            with gzip.open(filepath, "rt", encoding="utf-8") as f:
                for line in f:
                    data = json.loads(line)
                    samples.append(IntervalSample.model_validate(data))
        else:
            with open(filepath, "r", encoding="utf-8") as f:
                for line in f:
                    data = json.loads(line)
                    samples.append(IntervalSample.model_validate(data))

        return samples

    def write_metadata(self, name: str, data: dict[str, Any]) -> str:
        """Write arbitrary metadata."""
        filepath = self.base_path / f"{name}.json.gz"

        with gzip.open(filepath, "wt", encoding="utf-8") as f:
            json.dump(data, f, indent=2)

        return str(filepath.relative_to(self.base_path.parent))

    def read_metadata(self, path: str) -> dict[str, Any]:
        """Read metadata file."""
        filepath = self.base_path.parent / path

        with gzip.open(filepath, "rt", encoding="utf-8") as f:
            return json.load(f)

    def write_log(self, module_name: str, run_id: int, log_content: str) -> str:
        """Write module execution log."""
        filepath = self.base_path / f"log_{run_id}_{module_name}.txt"

        with open(filepath, "w", encoding="utf-8") as f:
            f.write(log_content)

        return str(filepath.relative_to(self.base_path.parent))

    def list_artifacts(self, pattern: str = "*") -> list[Path]:
        """List all artifacts matching pattern."""
        return list(self.base_path.glob(pattern))

    def cleanup_old_artifacts(self, keep_latest: int = 10) -> int:
        """Clean up old artifacts, keeping only the latest N."""
        traces = sorted(self.base_path.glob("trace_*.jsonl*"), key=lambda p: p.stat().st_mtime)

        to_delete = traces[:-keep_latest] if len(traces) > keep_latest else []

        for path in to_delete:
            path.unlink()

        return len(to_delete)
