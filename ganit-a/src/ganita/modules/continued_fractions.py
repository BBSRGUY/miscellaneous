"""
Continued fractions discovery module.

Explores various continued fraction templates.
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


class ContinuedFractionsModule(BaseModule):
    """Discovers constants via continued fractions."""

    name = "continued_fractions"
    description = "Explores continued fraction templates"

    def explore(self) -> list[Discovery]:
        """Run continued fractions exploration."""
        self._log_progress("Starting continued fractions exploration")

        discoveries: list[Discovery] = []

        grid_spec = self.config.get("parameter_grid", self._default_grid())
        param_grid = self._create_parameter_grid(grid_spec)

        self._log_progress(f"Generated {len(param_grid)} parameter combinations")

        for params in param_grid:
            if self._should_skip_params(params):
                continue

            template_type = params.get("template", "simple")

            try:
                if template_type == "simple":
                    disc = self._explore_simple_cf(params)
                elif template_type == "linear":
                    disc = self._explore_linear_cf(params)
                elif template_type == "quadratic":
                    disc = self._explore_quadratic_cf(params)
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
            "template": ["simple", "linear", "quadratic"],
            "a0": {"type": "range", "start": 0, "end": 3, "step": 1},
            "a_coeff": {"type": "range", "start": 1, "end": 4, "step": 1},
            "b_coeff": {"type": "range", "start": 1, "end": 4, "step": 1},
        }

    def _explore_simple_cf(self, params: dict[str, any]) -> Discovery | None:
        """
        Simple CF: a0 + b1/(a1 + b2/(a2 + ...))
        where a_n = a_coeff, b_n = b_coeff
        """
        a0 = params.get("a0", 1)
        a_coeff = params.get("a_coeff", 1)
        b_coeff = params.get("b_coeff", 1)

        def a_func(n: int) -> mp.mpf:
            if n == 0:
                return mp.mpf(a0)
            return mp.mpf(a_coeff)

        def b_func(n: int) -> mp.mpf:
            return mp.mpf(b_coeff)

        result = self.evaluator.continued_fraction(a_func, b_func, params)

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

    def _explore_linear_cf(self, params: dict[str, any]) -> Discovery | None:
        """
        Linear CF: a_n = a0 + a1*n, b_n = b0 + b1*n
        """
        a0 = params.get("a0", 1)
        a1 = params.get("a1", 0)
        b0 = params.get("b0", 1)
        b1 = params.get("b1", 1)

        def a_func(n: int) -> mp.mpf:
            return mp.mpf(a0 + a1 * n)

        def b_func(n: int) -> mp.mpf:
            return mp.mpf(b0 + b1 * n)

        result = self.evaluator.continued_fraction(a_func, b_func, params)

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

    def _explore_quadratic_cf(self, params: dict[str, any]) -> Discovery | None:
        """
        Quadratic CF: a_n = n^2, b_n = const
        """
        b_coeff = params.get("b_coeff", 1)
        a0 = params.get("a0", 0)

        def a_func(n: int) -> mp.mpf:
            if n == 0:
                return mp.mpf(a0)
            return mp.mpf(n * n)

        def b_func(n: int) -> mp.mpf:
            return mp.mpf(b_coeff)

        result = self.evaluator.continued_fraction(a_func, b_func, params)

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
            features.efficiency_score = max(0.0, 1.0 - final_n / 100000)

        return features
