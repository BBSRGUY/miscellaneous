"""
Fixed points and nested radicals discovery module.
"""

from __future__ import annotations

import logging

import mpmath as mp

from ganita.models import (
    ConfidenceStage,
    Discovery,
    EnvironmentSnapshot,
    Evidence,
    RankFeatures,
)
from ganita.modules.base import BaseModule

logger = logging.getLogger(__name__)


class FixedPointsModule(BaseModule):
    """Discovers constants via fixed point iteration and nested radicals."""

    name = "fixed_points"
    description = "Explores fixed point equations and nested radicals"

    def explore(self) -> list[Discovery]:
        """Run fixed points exploration."""
        self._log_progress("Starting fixed points exploration")

        discoveries: list[Discovery] = []

        grid_spec = self.config.get("parameter_grid", self._default_grid())
        param_grid = self._create_parameter_grid(grid_spec)

        self._log_progress(f"Generated {len(param_grid)} parameter combinations")

        for params in param_grid:
            if self._should_skip_params(params):
                continue

            template_type = params.get("template", "cosine")

            try:
                if template_type == "cosine":
                    disc = self._explore_cosine_fixed_point(params)
                elif template_type == "exponential":
                    disc = self._explore_exponential_fixed_point(params)
                elif template_type == "nested_sqrt":
                    disc = self._explore_nested_sqrt(params)
                elif template_type == "nested_power":
                    disc = self._explore_nested_power(params)
                else:
                    continue

                if disc:
                    discoveries.append(disc)

            except Exception as e:
                logger.warning(f"Error exploring {template_type} with params {params}: {e}")

        self._log_progress(f"Completed exploration with {len(discoveries)} discoveries")
        return discoveries

    def _default_grid(self) -> dict[str, any]:
        """Default parameter grid."""
        return {
            "template": ["cosine", "exponential", "nested_sqrt", "nested_power"],
            "scale": [0.5, 1.0, 2.0],
            "shift": [0.0, 1.0],
            "a": {"type": "range", "start": 1, "end": 4, "step": 1},
        }

    def _explore_cosine_fixed_point(self, params: dict[str, any]) -> Discovery | None:
        """
        Find fixed point of x = scale * cos(x + shift)
        """
        scale = params.get("scale", 1.0)
        shift = params.get("shift", 0.0)

        def f(x: mp.mpf) -> mp.mpf:
            return scale * mp.cos(x + shift)

        initial = mp.mpf(0.5)
        result = self.evaluator.fixed_point(f, initial, params)

        if not result.samples:
            return None

        evidence = Evidence(
            interval_trace=result.samples,
            accel_used=result.accel_used,
            cert_strategy=result.cert_strategy,
        )

        discovery = Discovery(
            value_repr=result.final.center,
            method=self.name,
            stage=ConfidenceStage.CERTIFIED if result.converged else ConfidenceStage.OBSERVED,
            params=params,
            env=EnvironmentSnapshot.create(),
            evidence=evidence,
        )

        if result.converged:
            recognition = self.recognizer.recognize(discovery.value_repr)
            if recognition and recognition.success:
                discovery.stage = ConfidenceStage.RECOGNIZED
                discovery.evidence.recognizer_summary = recognition

        discovery.rank_features = self._compute_rank_features(discovery)

        return discovery

    def _explore_exponential_fixed_point(self, params: dict[str, any]) -> Discovery | None:
        """
        Find fixed point of x = scale * exp(-x)
        """
        scale = params.get("scale", 1.0)

        def f(x: mp.mpf) -> mp.mpf:
            return scale * mp.exp(-x)

        initial = mp.mpf(0.5)
        result = self.evaluator.fixed_point(f, initial, params)

        if not result.samples:
            return None

        evidence = Evidence(
            interval_trace=result.samples,
            accel_used=result.accel_used,
            cert_strategy=result.cert_strategy,
        )

        discovery = Discovery(
            value_repr=result.final.center,
            method=self.name,
            stage=ConfidenceStage.CERTIFIED if result.converged else ConfidenceStage.OBSERVED,
            params=params,
            env=EnvironmentSnapshot.create(),
            evidence=evidence,
        )

        if result.converged:
            recognition = self.recognizer.recognize(discovery.value_repr)
            if recognition and recognition.success:
                discovery.stage = ConfidenceStage.RECOGNIZED
                discovery.evidence.recognizer_summary = recognition

        discovery.rank_features = self._compute_rank_features(discovery)

        return discovery

    def _explore_nested_sqrt(self, params: dict[str, any]) -> Discovery | None:
        """
        Explore nested radical: sqrt(a + sqrt(a + sqrt(a + ...)))
        """
        a = params.get("a", 2)

        def generator(n: int) -> mp.mpf:
            return mp.mpf(a)

        result = self.evaluator.nested_radical(generator, params)

        if not result.samples:
            return None

        evidence = Evidence(
            interval_trace=result.samples,
            accel_used=result.accel_used,
            cert_strategy=result.cert_strategy,
        )

        discovery = Discovery(
            value_repr=result.final.center,
            method=self.name,
            stage=ConfidenceStage.CERTIFIED if result.converged else ConfidenceStage.OBSERVED,
            params=params,
            env=EnvironmentSnapshot.create(),
            evidence=evidence,
        )

        if result.converged:
            recognition = self.recognizer.recognize(discovery.value_repr)
            if recognition and recognition.success:
                discovery.stage = ConfidenceStage.RECOGNIZED
                discovery.evidence.recognizer_summary = recognition

        discovery.rank_features = self._compute_rank_features(discovery)

        return discovery

    def _explore_nested_power(self, params: dict[str, any]) -> Discovery | None:
        """
        Explore x^x^x^... (tetration limit)
        """
        base = params.get("base", 1.5)

        # Only converges for e^(-e) <= base <= e^(1/e)
        lower_bound = float(mp.exp(-mp.e))
        upper_bound = float(mp.exp(1 / mp.e))

        if not (lower_bound <= base <= upper_bound):
            return None

        def f(x: mp.mpf) -> mp.mpf:
            return mp.power(base, x)

        initial = mp.mpf(1.5)
        result = self.evaluator.fixed_point(f, initial, params, max_iter=1000)

        if not result.samples:
            return None

        evidence = Evidence(
            interval_trace=result.samples,
            accel_used=result.accel_used,
            cert_strategy=result.cert_strategy,
        )

        discovery = Discovery(
            value_repr=result.final.center,
            method=self.name,
            stage=ConfidenceStage.CERTIFIED if result.converged else ConfidenceStage.OBSERVED,
            params=params,
            env=EnvironmentSnapshot.create(),
            evidence=evidence,
        )

        if result.converged:
            recognition = self.recognizer.recognize(discovery.value_repr)
            if recognition and recognition.success:
                discovery.stage = ConfidenceStage.RECOGNIZED
                discovery.evidence.recognizer_summary = recognition

        discovery.rank_features = self._compute_rank_features(discovery)

        return discovery

    def _compute_rank_features(self, discovery: Discovery) -> RankFeatures:
        """Compute ranking features."""
        features = RankFeatures()

        if discovery.ulp_bits > 0:
            target_bits = 2000
            features.interval_quality = min(1.0, discovery.ulp_bits / target_bits)

        conv_rate = discovery.evidence.convergence_rate
        if conv_rate is not None:
            features.convergence_smoothness = min(1.0, conv_rate / 10.0)

        if discovery.stage == ConfidenceStage.RECOGNIZED:
            rec_sum = discovery.evidence.recognizer_summary
            if rec_sum and rec_sum.height:
                features.recognition_score = max(0.0, 1.0 - rec_sum.height / 10000)
        elif discovery.stage == ConfidenceStage.NOVEL_CANDIDATE:
            features.recognition_score = 0.9

        if discovery.evidence.interval_trace:
            final_n = discovery.evidence.interval_trace[-1].n
            features.efficiency_score = max(0.0, 1.0 - final_n / 10000)

        return features
