"""
Core data models for mathematical discoveries.

All models use Pydantic for validation and serialization.
"""

from __future__ import annotations

import hashlib
import json
import platform
import sys
from datetime import datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, Field, computed_field


class ConfidenceStage(str, Enum):
    """Life-cycle stage of a discovery."""

    OBSERVED = "observed"  # Numeric value observed but not yet certified
    CERTIFIED = "certified"  # Interval bounds rigorously certified
    RECOGNIZED = "recognized"  # Matched to known constant or simple relation
    CROSS_VALIDATED = "cross_validated"  # Reproduced via independent methods
    NOVEL_CANDIDATE = "novel_candidate"  # Exhaustive recognition failed; likely novel


class IntervalSample(BaseModel):
    """A single interval sample during convergence."""

    n: int = Field(..., description="Iteration/term count")
    center: str = Field(..., description="Center value as decimal string")
    radius: str = Field(..., description="Radius/uncertainty as decimal string")
    meta: dict[str, Any] = Field(default_factory=dict, description="Method-specific metadata")

    @computed_field
    @property
    def log10_radius(self) -> float:
        """Log10 of radius for plotting."""
        try:
            from mpmath import log10, mpf

            return float(log10(mpf(self.radius)))
        except:
            return 0.0


class RecognizerSummary(BaseModel):
    """Summary of PSLQ/LLL recognition attempts."""

    basis_name: str
    success: bool
    relation_tex: str | None = None
    relation_json: dict[str, Any] | None = None
    height: int | None = None  # Coefficient height
    residual_log10: float | None = None
    precision_used: int
    stability_verified: bool = False


class Evidence(BaseModel):
    """Complete evidence trail for a discovery."""

    interval_trace: list[IntervalSample] = Field(
        default_factory=list, description="Sequence of interval samples"
    )
    accel_used: list[str] = Field(
        default_factory=list, description="Acceleration methods applied"
    )
    cert_strategy: str = Field(default="none", description="Certification strategy")
    recognizer_summary: RecognizerSummary | None = None
    cross_validation_methods: list[str] = Field(default_factory=list)

    @computed_field
    @property
    def convergence_rate(self) -> float | None:
        """Estimate convergence rate from trace."""
        if len(self.interval_trace) < 3:
            return None

        try:
            from mpmath import log10, mpf

            samples = self.interval_trace[-10:]  # Last 10 samples
            if len(samples) < 2:
                return None

            log_radii = [float(log10(mpf(s.radius))) for s in samples]
            n_vals = [s.n for s in samples]

            # Linear fit
            n_mean = sum(n_vals) / len(n_vals)
            lr_mean = sum(log_radii) / len(log_radii)

            numerator = sum((n - n_mean) * (lr - lr_mean) for n, lr in zip(n_vals, log_radii))
            denominator = sum((n - n_mean) ** 2 for n in n_vals)

            if denominator > 0:
                return abs(numerator / denominator)
            return None
        except:
            return None


class RankFeatures(BaseModel):
    """Features used for ranking discoveries."""

    interval_quality: float = Field(
        default=0.0, ge=0.0, le=1.0, description="Quality based on final precision"
    )
    convergence_smoothness: float = Field(
        default=0.0, ge=0.0, le=1.0, description="Smoothness of convergence"
    )
    recognition_score: float = Field(
        default=0.0, ge=0.0, le=1.0, description="Recognition/novelty score"
    )
    cross_validation_score: float = Field(
        default=0.0, ge=0.0, le=1.0, description="Cross-validation bonus"
    )
    efficiency_score: float = Field(
        default=0.0, ge=0.0, le=1.0, description="Computational efficiency"
    )

    @computed_field
    @property
    def total_score(self) -> float:
        """Weighted total score."""
        weights = {
            "interval_quality": 0.30,
            "convergence_smoothness": 0.15,
            "recognition_score": 0.20,
            "cross_validation_score": 0.25,
            "efficiency_score": 0.10,
        }
        return (
            weights["interval_quality"] * self.interval_quality
            + weights["convergence_smoothness"] * self.convergence_smoothness
            + weights["recognition_score"] * self.recognition_score
            + weights["cross_validation_score"] * self.cross_validation_score
            + weights["efficiency_score"] * self.efficiency_score
        )


class EnvironmentSnapshot(BaseModel):
    """Snapshot of execution environment for reproducibility."""

    python_version: str = Field(default_factory=lambda: sys.version)
    platform: str = Field(default_factory=platform.platform)
    timestamp: str = Field(default_factory=lambda: datetime.utcnow().isoformat())
    code_hash: str | None = None
    mpmath_version: str | None = None
    sympy_version: str | None = None
    random_seed: int | None = None

    @staticmethod
    def create(code_hash: str | None = None, seed: int | None = None) -> EnvironmentSnapshot:
        """Create environment snapshot with version info."""
        try:
            import mpmath
            import sympy

            return EnvironmentSnapshot(
                code_hash=code_hash,
                random_seed=seed,
                mpmath_version=mpmath.__version__,
                sympy_version=sympy.__version__,
            )
        except ImportError:
            return EnvironmentSnapshot(code_hash=code_hash, random_seed=seed)


class Discovery(BaseModel):
    """A mathematical discovery candidate."""

    discovery_id: int | None = None
    value_repr: str = Field(..., description="Canonical decimal representation")
    method: str = Field(..., description="Discovery module name")
    stage: ConfidenceStage = Field(default=ConfidenceStage.OBSERVED)
    rank_features: RankFeatures = Field(default_factory=RankFeatures)
    params: dict[str, Any] = Field(default_factory=dict, description="Generation parameters")
    env: EnvironmentSnapshot = Field(default_factory=EnvironmentSnapshot)
    evidence: Evidence = Field(default_factory=Evidence)
    created_at: str = Field(default_factory=lambda: datetime.utcnow().isoformat())

    @computed_field
    @property
    def rank_score(self) -> float:
        """Overall rank score."""
        return self.rank_features.total_score

    @computed_field
    @property
    def ulp_bits(self) -> int:
        """Effective precision in bits."""
        if not self.evidence.interval_trace:
            return 0
        try:
            from mpmath import log, mpf

            final = self.evidence.interval_trace[-1]
            radius = mpf(final.radius)
            if radius > 0:
                return int(-float(log(radius, 2)))
            return 0
        except:
            return 0

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for storage."""
        return self.model_dump(mode="json")

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Discovery:
        """Load from dictionary."""
        return cls.model_validate(data)

    def compute_hash(self) -> str:
        """Compute deterministic hash of this discovery."""
        key_data = {
            "value": self.value_repr,
            "method": self.method,
            "params": self.params,
        }
        content = json.dumps(key_data, sort_keys=True)
        return hashlib.sha256(content.encode()).hexdigest()[:16]


class ModuleConfig(BaseModel):
    """Configuration for a discovery module."""

    name: str
    enabled: bool = True
    budget_minutes: float = 30.0
    max_terms: int = 100000
    parameter_grid: dict[str, Any] = Field(default_factory=dict)


class RunConfig(BaseModel):
    """Configuration for a discovery run."""

    target_precision_digits: int = 600
    wallclock_limit_minutes: float = 120.0
    random_seed: int = 42
    max_workers: int = 8
    modules: list[ModuleConfig] = Field(default_factory=list)

    evaluator: dict[str, Any] = Field(
        default_factory=lambda: {
            "library": "mpmath",
            "precision_ramp": [128, 256, 512, 1024, 2048],
            "stop_radius": "1e-500",
            "max_terms": 100000,
        }
    )

    recognizer: dict[str, Any] = Field(
        default_factory=lambda: {
            "bases": [
                {
                    "name": "small_v1",
                    "symbols": ["1", "PI", "LOG2", "ZETA3", "CATALAN", "EULER_GAMMA"],
                    "max_height": 2000,
                }
            ],
            "pslq_precision_digits": [200, 400, 800],
        }
    )

    ranking: dict[str, float] = Field(
        default_factory=lambda: {
            "q1": 0.30,
            "q2": 0.15,
            "q3": 0.20,
            "q4": 0.25,
            "q5": 0.10,
        }
    )


class AuditLog(BaseModel):
    """Audit log entry for governance."""

    audit_id: int | None = None
    ts: str = Field(default_factory=lambda: datetime.utcnow().isoformat())
    actor: str = "system"
    action: str
    subject_type: str
    subject_id: int
    payload: dict[str, Any] = Field(default_factory=dict)
