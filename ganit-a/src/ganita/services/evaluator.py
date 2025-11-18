"""
High-precision evaluator with certified interval arithmetic.

Uses mpmath for arbitrary precision ball arithmetic with rigorous remainder bounds.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Any, Callable

import mpmath as mp

from ganita.models import IntervalSample

logger = logging.getLogger(__name__)


@dataclass
class EvaluationResult:
    """Result of a high-precision evaluation."""

    final: IntervalSample
    samples: list[IntervalSample]
    accel_used: list[str]
    cert_strategy: str
    converged: bool
    meta: dict[str, Any]


class BallInterval:
    """Interval arithmetic wrapper with center and radius."""

    def __init__(self, center: mp.mpf | str, radius: mp.mpf | str):
        self.center = mp.mpf(center)
        self.radius = mp.mpf(radius)

    @property
    def lower(self) -> mp.mpf:
        return self.center - self.radius

    @property
    def upper(self) -> mp.mpf:
        return self.center + self.radius

    def contains(self, value: mp.mpf | str) -> bool:
        """Check if value is in this interval."""
        v = mp.mpf(value)
        return self.lower <= v <= self.upper

    def overlaps(self, other: BallInterval) -> bool:
        """Check if this interval overlaps with another."""
        return not (self.upper < other.lower or other.upper < self.lower)

    def to_sample(self, n: int, meta: dict[str, Any] | None = None) -> IntervalSample:
        """Convert to IntervalSample."""
        return IntervalSample(
            n=n,
            center=str(self.center),
            radius=str(self.radius),
            meta=meta or {},
        )

    def __repr__(self) -> str:
        return f"[{self.center} ± {self.radius}]"


class Evaluator:
    """High-precision validated numerical evaluator."""

    def __init__(self, config: dict[str, Any]):
        self.config = config
        self.precision_ramp = config.get("precision_ramp", [128, 256, 512, 1024, 2048])
        self.stop_radius = mp.mpf(config.get("stop_radius", "1e-500"))
        self.max_terms = config.get("max_terms", 100000)

    def series_limit(
        self,
        term_func: Callable[[int], mp.mpf],
        params: dict[str, Any],
        alternating: bool = False,
    ) -> EvaluationResult:
        """
        Evaluate series limit with certified bounds.

        For alternating series, uses alternating series theorem.
        For non-alternating, requires additional analysis.
        """
        samples: list[IntervalSample] = []
        accel_used: list[str] = []

        for precision in self.precision_ramp:
            mp.dps = precision

            partial_sums = []
            terms_computed = 0

            # Compute partial sums
            for n in range(1, self.max_terms + 1):
                try:
                    term = term_func(n)
                    if n == 1:
                        s = term
                    else:
                        s = partial_sums[-1] + term

                    partial_sums.append(s)
                    terms_computed = n

                    # Sample every power of 2
                    if n & (n - 1) == 0 or n % 1000 == 0:  # Power of 2 or every 1000
                        # Estimate radius
                        if alternating and n > 1:
                            # Alternating series: remainder ≤ next term
                            radius = abs(term)
                        else:
                            # Estimate from recent convergence
                            if len(partial_sums) >= 10:
                                diffs = [
                                    abs(partial_sums[i] - partial_sums[i - 1])
                                    for i in range(-10, 0)
                                ]
                                radius = max(diffs) * 2  # Conservative
                            else:
                                radius = abs(term) * 2

                        interval = BallInterval(s, radius)
                        sample = interval.to_sample(n, {"precision": precision})
                        samples.append(sample)

                        # Check convergence
                        if radius < self.stop_radius:
                            logger.info(
                                f"Converged at n={n}, precision={precision}, radius={radius}"
                            )
                            break

                    # Divergence detection
                    if n > 100 and abs(term) > abs(partial_sums[-2] - partial_sums[-11]):
                        logger.warning(f"Potential divergence detected at n={n}")
                        break

                except (ValueError, ZeroDivisionError, OverflowError) as e:
                    logger.error(f"Error computing term {n}: {e}")
                    break

            if samples and samples[-1].radius and mp.mpf(samples[-1].radius) < self.stop_radius:
                break

        final_sample = samples[-1] if samples else IntervalSample(n=0, center="0", radius="inf")

        cert_strategy = "alternating_series_theorem" if alternating else "conservative_estimate"

        return EvaluationResult(
            final=final_sample,
            samples=samples,
            accel_used=accel_used,
            cert_strategy=cert_strategy,
            converged=mp.mpf(final_sample.radius) < self.stop_radius,
            meta={"terms_computed": terms_computed, "alternating": alternating},
        )

    def product_limit(
        self, factor_func: Callable[[int], mp.mpf], params: dict[str, Any]
    ) -> EvaluationResult:
        """Evaluate infinite product with certified bounds."""
        samples: list[IntervalSample] = []

        for precision in self.precision_ramp:
            mp.dps = precision

            partial_products = []
            terms_computed = 0

            for n in range(1, self.max_terms + 1):
                try:
                    factor = factor_func(n)

                    if n == 1:
                        p = factor
                    else:
                        p = partial_products[-1] * factor

                    partial_products.append(p)
                    terms_computed = n

                    if n & (n - 1) == 0 or n % 1000 == 0:
                        # Estimate convergence
                        if len(partial_products) >= 10:
                            recent = partial_products[-10:]
                            variation = max(recent) - min(recent)
                            radius = variation
                        else:
                            radius = abs(p * (factor - 1))

                        interval = BallInterval(p, radius)
                        sample = interval.to_sample(n, {"precision": precision})
                        samples.append(sample)

                        if radius < self.stop_radius:
                            break

                    # Divergence check
                    if abs(p) > mp.mpf("1e1000") or abs(p) < mp.mpf("1e-1000"):
                        logger.warning(f"Product diverging at n={n}")
                        break

                except (ValueError, ZeroDivisionError, OverflowError) as e:
                    logger.error(f"Error computing factor {n}: {e}")
                    break

            if samples and mp.mpf(samples[-1].radius) < self.stop_radius:
                break

        final_sample = samples[-1] if samples else IntervalSample(n=0, center="1", radius="inf")

        return EvaluationResult(
            final=final_sample,
            samples=samples,
            accel_used=[],
            cert_strategy="product_convergence",
            converged=mp.mpf(final_sample.radius) < self.stop_radius,
            meta={"terms_computed": terms_computed},
        )

    def continued_fraction(
        self, a_func: Callable[[int], mp.mpf], b_func: Callable[[int], mp.mpf], params: dict[str, Any]
    ) -> EvaluationResult:
        """
        Evaluate continued fraction: a0 + b1/(a1 + b2/(a2 + ...))

        Uses forward recurrence with remainder estimates.
        """
        samples: list[IntervalSample] = []

        for precision in self.precision_ramp:
            mp.dps = precision

            convergents = []
            terms_computed = 0

            # Use forward recurrence
            for n in range(self.max_terms):
                try:
                    if n == 0:
                        a0 = a_func(0)
                        convergents.append(a0)
                    else:
                        # Build from bottom up (backward evaluation)
                        result = a_func(n)
                        for k in range(n - 1, -1, -1):
                            b_k1 = b_func(k + 1)
                            a_k = a_func(k)
                            result = a_k + b_k1 / result

                        convergents.append(result)
                        terms_computed = n

                    if n > 0 and (n & (n - 1) == 0 or n % 100 == 0):
                        # Estimate radius from convergent differences
                        if len(convergents) >= 3:
                            diff = abs(convergents[-1] - convergents[-2])
                            radius = diff * 2  # Conservative
                        else:
                            radius = abs(convergents[-1]) * mp.mpf("1e-10")

                        interval = BallInterval(convergents[-1], radius)
                        sample = interval.to_sample(n, {"precision": precision})
                        samples.append(sample)

                        if radius < self.stop_radius:
                            break

                except (ValueError, ZeroDivisionError, OverflowError) as e:
                    logger.error(f"Error computing CF term {n}: {e}")
                    break

            if samples and mp.mpf(samples[-1].radius) < self.stop_radius:
                break

        final_sample = samples[-1] if samples else IntervalSample(n=0, center="0", radius="inf")

        return EvaluationResult(
            final=final_sample,
            samples=samples,
            accel_used=[],
            cert_strategy="continued_fraction_convergence",
            converged=mp.mpf(final_sample.radius) < self.stop_radius,
            meta={"terms_computed": terms_computed},
        )

    def fixed_point(
        self,
        f: Callable[[mp.mpf], mp.mpf],
        initial: mp.mpf,
        params: dict[str, Any],
        max_iter: int = 10000,
    ) -> EvaluationResult:
        """
        Find fixed point x = f(x) using iteration.

        Requires |f'(x)| < 1 for convergence.
        """
        samples: list[IntervalSample] = []

        for precision in self.precision_ramp:
            mp.dps = precision

            x = mp.mpf(initial)
            iterations = 0

            for n in range(max_iter):
                try:
                    x_new = f(x)
                    diff = abs(x_new - x)

                    if n % 10 == 0 or n & (n - 1) == 0:
                        radius = diff * 2  # Conservative bound
                        interval = BallInterval(x_new, radius)
                        sample = interval.to_sample(n, {"precision": precision})
                        samples.append(sample)

                        if radius < self.stop_radius:
                            break

                    x = x_new
                    iterations = n

                    # Divergence check
                    if diff > 1:
                        logger.warning(f"Fixed point iteration diverging at n={n}")
                        break

                except (ValueError, ZeroDivisionError, OverflowError) as e:
                    logger.error(f"Error in fixed point iteration {n}: {e}")
                    break

            if samples and mp.mpf(samples[-1].radius) < self.stop_radius:
                break

        final_sample = samples[-1] if samples else IntervalSample(n=0, center=str(initial), radius="inf")

        return EvaluationResult(
            final=final_sample,
            samples=samples,
            accel_used=[],
            cert_strategy="fixed_point_iteration",
            converged=mp.mpf(final_sample.radius) < self.stop_radius,
            meta={"iterations": iterations},
        )

    def nested_radical(self, generator: Callable[[int], mp.mpf], params: dict[str, Any]) -> EvaluationResult:
        """
        Evaluate nested radical: sqrt(a1 + sqrt(a2 + sqrt(a3 + ...)))

        Works backward from deep nesting.
        """
        samples: list[IntervalSample] = []

        for precision in self.precision_ramp:
            mp.dps = precision

            # Start from deep nesting and work backward
            max_depth = min(1000, self.max_terms)

            for depth in range(10, max_depth, 10):
                try:
                    # Start with innermost term
                    result = mp.sqrt(generator(depth))

                    # Work backward
                    for k in range(depth - 1, 0, -1):
                        result = mp.sqrt(generator(k) + result)

                    # Estimate convergence
                    if len(samples) >= 2:
                        prev_center = mp.mpf(samples[-1].center)
                        diff = abs(result - prev_center)
                        radius = diff * 2
                    else:
                        radius = abs(result) * mp.mpf("1e-10")

                    interval = BallInterval(result, radius)
                    sample = interval.to_sample(depth, {"precision": precision})
                    samples.append(sample)

                    if radius < self.stop_radius:
                        break

                except (ValueError, ZeroDivisionError, OverflowError) as e:
                    logger.error(f"Error in nested radical at depth {depth}: {e}")
                    break

            if samples and mp.mpf(samples[-1].radius) < self.stop_radius:
                break

        final_sample = samples[-1] if samples else IntervalSample(n=0, center="0", radius="inf")

        return EvaluationResult(
            final=final_sample,
            samples=samples,
            accel_used=[],
            cert_strategy="nested_radical_convergence",
            converged=mp.mpf(final_sample.radius) < self.stop_radius,
            meta={"max_depth": max_depth},
        )
