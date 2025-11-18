"""
Series and products discovery module.

Explores various series and product templates to discover constants.
"""

from __future__ import annotations

import logging
from typing import Any, Callable

import mpmath as mp

from ganita.models import (
    ConfidenceStage,
    Discovery,
    EnvironmentSnapshot,
    Evidence,
    RankFeatures,
)
from ganita.modules.base import BaseModule
from ganita.services.acceleration import Accelerator

logger = logging.getLogger(__name__)


class SeriesProductsModule(BaseModule):
    """Discovers constants via series and infinite products."""

    name = "series_products"
    description = "Explores series and infinite product templates"

    def explore(self) -> list[Discovery]:
        """Run series and products exploration."""
        self._log_progress("Starting series and products exploration")

        discoveries: list[Discovery] = []

        # Get parameter grid from config
        grid_spec = self.config.get("parameter_grid", self._default_grid())
        param_grid = self._create_parameter_grid(grid_spec)

        self._log_progress(f"Generated {len(param_grid)} parameter combinations")

        # Explore templates
        for params in param_grid:
            if self._should_skip_params(params):
                continue

            # Determine template type
            template_type = params.get("template", "dirichlet")

            try:
                if template_type == "dirichlet":
                    disc = self._explore_dirichlet(params)
                elif template_type == "polylog":
                    disc = self._explore_polylog(params)
                elif template_type == "euler_product":
                    disc = self._explore_euler_product(params)
                elif template_type == "rational_series":
                    disc = self._explore_rational_series(params)
                elif template_type == "fibonacci_like":
                    disc = self._explore_fibonacci_like(params)
                else:
                    continue

                if disc:
                    discoveries.append(disc)

            except Exception as e:
                logger.warning(f"Error exploring {template_type} with params {params}: {e}")

        self._log_progress(f"Completed exploration with {len(discoveries)} discoveries")
        return discoveries

    def _default_grid(self) -> dict[str, Any]:
        """Default parameter grid."""
        return {
            "template": [
                "dirichlet",
                "polylog",
                "rational_series",
                "fibonacci_like",
            ],
            "p": {"type": "range", "start": 2, "end": 6, "step": 1},
            "a": {"type": "range", "start": 1, "end": 4, "step": 1},
            "b": {"type": "range", "start": 0, "end": 3, "step": 1},
            "sign": [1, -1],
        }

    def _explore_dirichlet(self, params: dict[str, Any]) -> Discovery | None:
        """
        Explore Dirichlet-like series: Σ sign^n / (a*n + b)^p
        """
        p = params.get("p", 2)
        a = params.get("a", 1)
        b = params.get("b", 0)
        sign = params.get("sign", 1)

        # Skip invalid parameters
        if b == 0 and a == 0:
            return None
        if p <= 0:
            return None

        alternating = sign == -1

        def term_func(n: int) -> mp.mpf:
            denominator = a * n + b
            if denominator == 0:
                return mp.mpf(0)
            return (sign**n) * mp.power(denominator, -p)

        # Evaluate
        result = self.evaluator.series_limit(term_func, params, alternating=alternating)

        if not result.samples:
            return None

        # Try acceleration
        if len(result.samples) >= 10:
            sequence = [mp.mpf(s.center) for s in result.samples[-20:]]
            method, accel_value = Accelerator.choose_best_method(sequence, alternating)
            if method != "none":
                result.accel_used.append(method)
                # Update final value
                result.final.center = str(accel_value)

        # Build discovery
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

        # Try recognition
        if result.converged:
            recognition = self.recognizer.recognize(discovery.value_repr)
            if recognition and recognition.success:
                discovery.stage = ConfidenceStage.RECOGNIZED
                discovery.evidence.recognizer_summary = recognition

        # Compute rank features
        discovery.rank_features = self._compute_rank_features(discovery)

        return discovery

    def _explore_polylog(self, params: dict[str, Any]) -> Discovery | None:
        """
        Explore polylogarithm series: Σ x^n / n^p
        """
        p = params.get("p", 2)
        # Use rational x values
        x_num = params.get("x_num", 1)
        x_den = params.get("x_den", 2)

        if x_den == 0:
            return None

        x = mp.mpf(x_num) / mp.mpf(x_den)

        # Only valid for |x| <= 1
        if abs(x) > 1:
            return None

        def term_func(n: int) -> mp.mpf:
            if n == 0:
                return mp.mpf(0)
            return mp.power(x, n) / mp.power(n, p)

        result = self.evaluator.series_limit(term_func, params, alternating=False)

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

    def _explore_euler_product(self, params: dict[str, Any]) -> Discovery | None:
        """
        Explore Euler products over primes: Π (1 - p^(-s))^(-1)
        """
        s = params.get("s", 2)

        if s <= 1:
            return None

        # Generate primes
        def is_prime(n: int) -> bool:
            if n < 2:
                return False
            if n == 2:
                return True
            if n % 2 == 0:
                return False
            for i in range(3, int(n**0.5) + 1, 2):
                if n % i == 0:
                    return False
            return True

        primes = [p for p in range(2, 10000) if is_prime(p)]

        def factor_func(n: int) -> mp.mpf:
            if n >= len(primes):
                return mp.mpf(1)
            p = primes[n]
            return 1 / (1 - mp.power(p, -s))

        result = self.evaluator.product_limit(factor_func, params)

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

    def _explore_rational_series(self, params: dict[str, Any]) -> Discovery | None:
        """
        Explore series with rational terms: Σ 1/(n^2 + a*n + b)
        """
        a = params.get("a", 1)
        b = params.get("b", 1)

        def term_func(n: int) -> mp.mpf:
            denom = n * n + a * n + b
            if denom == 0:
                return mp.mpf(0)
            return mp.mpf(1) / mp.mpf(denom)

        result = self.evaluator.series_limit(term_func, params, alternating=False)

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

    def _explore_fibonacci_like(self, params: dict[str, Any]) -> Discovery | None:
        """
        Explore Fibonacci-like series: Σ 1/F_n^p where F_n is Fibonacci-like
        """
        p = params.get("p", 1)
        a = params.get("fib_a", 1)  # F_n = a*F_{n-1} + b*F_{n-2}
        b = params.get("fib_b", 1)

        if p <= 0:
            return None

        # Generate Fibonacci-like sequence
        fib = [0, 1]
        for i in range(2, 1000):
            fib.append(a * fib[-1] + b * fib[-2])

        def term_func(n: int) -> mp.mpf:
            if n >= len(fib) or fib[n] == 0:
                return mp.mpf(0)
            return mp.power(fib[n], -p)

        result = self.evaluator.series_limit(term_func, params, alternating=False)

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
        """Compute ranking features for a discovery."""
        features = RankFeatures()

        # Interval quality
        if discovery.ulp_bits > 0:
            # Normalize to [0, 1] based on target precision
            target_bits = 2000  # ~600 decimal digits
            features.interval_quality = min(1.0, discovery.ulp_bits / target_bits)

        # Convergence smoothness
        conv_rate = discovery.evidence.convergence_rate
        if conv_rate is not None:
            features.convergence_smoothness = min(1.0, conv_rate / 10.0)

        # Recognition score
        if discovery.stage == ConfidenceStage.RECOGNIZED:
            rec_sum = discovery.evidence.recognizer_summary
            if rec_sum and rec_sum.height:
                # Lower height = simpler relation = higher score
                features.recognition_score = max(0.0, 1.0 - rec_sum.height / 10000)
        elif discovery.stage == ConfidenceStage.NOVEL_CANDIDATE:
            features.recognition_score = 0.9  # Novel candidates get high score

        # Efficiency score (based on terms needed)
        if discovery.evidence.interval_trace:
            final_n = discovery.evidence.interval_trace[-1].n
            # Prefer faster convergence
            features.efficiency_score = max(0.0, 1.0 - final_n / 100000)

        return features
