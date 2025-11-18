"""
Acceleration methods for series convergence.

Implements Wynn epsilon, Aitken delta-squared, Euler-Maclaurin,
Van Wijngaarden, Richardson extrapolation, and Shanks transformation.
"""

from __future__ import annotations

import logging
from typing import Any

import mpmath as mp

logger = logging.getLogger(__name__)


class Accelerator:
    """Collection of series acceleration methods."""

    @staticmethod
    def wynn_epsilon(sequence: list[mp.mpf], return_all: bool = False) -> mp.mpf | list[mp.mpf]:
        """
        Wynn epsilon algorithm for accelerating convergence.

        Highly effective for alternating series and rational convergents.
        """
        n = len(sequence)
        if n < 3:
            return sequence[-1] if sequence else mp.mpf(0)

        # Build epsilon table
        eps = [[mp.mpf(0) for _ in range(n + 1)] for _ in range(n + 1)]

        # Initialize
        for i in range(n):
            eps[i][1] = sequence[i]

        # Fill table
        for i in range(n - 1):
            for j in range(2, n - i + 1):
                try:
                    denominator = eps[i + 1][j - 1] - eps[i][j - 1]
                    if abs(denominator) > mp.mpf("1e-1000"):
                        eps[i][j] = eps[i + 1][j - 2] + 1 / denominator
                    else:
                        eps[i][j] = eps[i][j - 1]
                except (ZeroDivisionError, OverflowError):
                    eps[i][j] = eps[i][j - 1]

        if return_all:
            # Return diagonal elements (most accelerated)
            diagonal = []
            for i in range(min(n, n)):
                if i < len(eps) and 2 * i + 1 < len(eps[0]):
                    diagonal.append(eps[0][2 * i + 1])
            return diagonal

        # Return best estimate (bottom-right even column)
        for j in range(n, 0, -1):
            if j % 2 == 0 and eps[0][j] != 0:
                return eps[0][j]

        return sequence[-1]

    @staticmethod
    def aitken_delta_squared(s0: mp.mpf, s1: mp.mpf, s2: mp.mpf) -> mp.mpf:
        """
        Aitken's delta-squared process for sequence acceleration.

        Given three consecutive terms, extrapolate to limit.
        """
        try:
            delta1 = s1 - s0
            delta2 = s2 - s1
            denominator = delta2 - delta1

            if abs(denominator) > mp.mpf("1e-1000"):
                return s0 - delta1**2 / denominator
            else:
                return s2
        except (ZeroDivisionError, OverflowError):
            return s2

    @staticmethod
    def shanks_transform(sequence: list[mp.mpf]) -> mp.mpf:
        """
        Shanks transformation for accelerating linearly convergent sequences.
        """
        n = len(sequence)
        if n < 3:
            return sequence[-1] if sequence else mp.mpf(0)

        # Apply Aitken to last three terms
        return Accelerator.aitken_delta_squared(sequence[-3], sequence[-2], sequence[-1])

    @staticmethod
    def richardson_extrapolation(
        values: list[tuple[mp.mpf, mp.mpf]], order: int = 2
    ) -> mp.mpf:
        """
        Richardson extrapolation for removing leading error terms.

        values: list of (h, f(h)) where h is step size
        order: order of extrapolation
        """
        if len(values) < 2:
            return values[0][1] if values else mp.mpf(0)

        # Sort by step size
        values = sorted(values, key=lambda x: x[0])

        # Build Richardson table
        r = [[v[1] for v in values]]

        for k in range(1, min(order + 1, len(values))):
            row = []
            for i in range(len(values) - k):
                try:
                    h_i = values[i][0]
                    h_ik = values[i + k][0]
                    ratio = (h_i / h_ik) ** (2 * k)

                    if ratio != 1:
                        val = (ratio * r[k - 1][i + 1] - r[k - 1][i]) / (ratio - 1)
                        row.append(val)
                    else:
                        row.append(r[k - 1][i + 1])
                except (ZeroDivisionError, OverflowError):
                    row.append(r[k - 1][i + 1] if i + 1 < len(r[k - 1]) else r[k - 1][i])
            r.append(row)

        # Return most extrapolated value
        return r[-1][0] if r[-1] else values[-1][1]

    @staticmethod
    def van_wijngaarden_sum(terms: list[mp.mpf]) -> mp.mpf:
        """
        Van Wijngaarden transformation for alternating series.

        Particularly effective for slowly converging alternating series.
        """
        if not terms:
            return mp.mpf(0)

        # Build transformation table
        table = [terms[:]]

        while len(table[-1]) > 1:
            prev_row = table[-1]
            new_row = []
            for i in range(len(prev_row) - 1):
                new_row.append((prev_row[i] + prev_row[i + 1]) / 2)
            table.append(new_row)

        # Sum the first element of each row
        result = mp.mpf(0)
        for row in table:
            if row:
                result += row[0]

        return result

    @staticmethod
    def euler_maclaurin_sum(
        term_func: Any,
        start: int,
        end: int,
        num_corrections: int = 5
    ) -> tuple[mp.mpf, mp.mpf]:
        """
        Euler-Maclaurin formula for series acceleration.

        Returns (sum, estimated_tail_bound)
        """
        # Direct sum
        direct_sum = mp.mpf(0)
        for n in range(start, end + 1):
            try:
                direct_sum += term_func(n)
            except (ValueError, ZeroDivisionError, OverflowError):
                break

        # Estimate tail using integral
        # For many series, tail ~ integral from end to infinity
        # This is a simplified version; full EM requires derivatives

        try:
            # Rough tail estimate: geometric decay assumption
            if end > start + 10:
                recent_terms = [abs(term_func(n)) for n in range(end - 10, end)]
                if recent_terms:
                    avg_term = sum(recent_terms) / len(recent_terms)
                    # Assume geometric decay
                    if avg_term > 0:
                        ratio = recent_terms[-1] / recent_terms[0] if recent_terms[0] != 0 else 0.5
                        if 0 < ratio < 1:
                            tail_bound = recent_terms[-1] / (1 - ratio)
                        else:
                            tail_bound = recent_terms[-1] * 10
                    else:
                        tail_bound = mp.mpf(0)
                else:
                    tail_bound = mp.mpf(0)
            else:
                tail_bound = abs(term_func(end)) * 10

        except (ValueError, ZeroDivisionError, OverflowError):
            tail_bound = mp.mpf("1e-10")

        return direct_sum, tail_bound

    @staticmethod
    def levin_transformation(sequence: list[mp.mpf], beta: float = 1.0) -> mp.mpf:
        """
        Levin u-transformation for sequence acceleration.

        Effective for sequences with known asymptotic behavior.
        """
        n = len(sequence)
        if n < 2:
            return sequence[-1] if sequence else mp.mpf(0)

        try:
            # Compute differences
            deltas = [sequence[i + 1] - sequence[i] for i in range(n - 1)]

            if not deltas:
                return sequence[-1]

            # Levin u-transform
            numerator = mp.mpf(0)
            denominator = mp.mpf(0)

            for k in range(len(deltas)):
                weight = (k + 1) ** beta / abs(deltas[k]) if deltas[k] != 0 else mp.mpf(0)
                numerator += weight * sequence[k + 1]
                denominator += weight

            if denominator > mp.mpf("1e-1000"):
                return numerator / denominator
            else:
                return sequence[-1]

        except (ZeroDivisionError, OverflowError):
            return sequence[-1]

    @staticmethod
    def choose_best_method(
        sequence: list[mp.mpf], alternating: bool = False
    ) -> tuple[str, mp.mpf]:
        """
        Automatically choose and apply the best acceleration method.

        Returns (method_name, accelerated_value)
        """
        if len(sequence) < 3:
            return ("none", sequence[-1] if sequence else mp.mpf(0))

        results = {}

        # Try Wynn epsilon (best for alternating and rational)
        try:
            results["wynn_epsilon"] = Accelerator.wynn_epsilon(sequence)
        except Exception as e:
            logger.debug(f"Wynn epsilon failed: {e}")

        # Try Shanks
        try:
            results["shanks"] = Accelerator.shanks_transform(sequence)
        except Exception as e:
            logger.debug(f"Shanks failed: {e}")

        # Try Van Wijngaarden for alternating
        if alternating:
            try:
                # Check if truly alternating
                signs = [mp.sign(sequence[i + 1] - sequence[i]) for i in range(len(sequence) - 1)]
                if len(set(signs)) > 1:  # Changes sign
                    results["van_wijngaarden"] = Accelerator.van_wijngaarden_sum(sequence)
            except Exception as e:
                logger.debug(f"Van Wijngaarden failed: {e}")

        # Try Levin
        try:
            results["levin"] = Accelerator.levin_transformation(sequence)
        except Exception as e:
            logger.debug(f"Levin failed: {e}")

        # Choose result closest to the last few terms (stability criterion)
        if not results:
            return ("none", sequence[-1])

        # Score by consistency with recent trend
        scores = {}
        recent_avg = sum(sequence[-5:]) / len(sequence[-5:])

        for method, value in results.items():
            scores[method] = abs(value - recent_avg)

        # Return method with smallest deviation
        best_method = min(scores, key=scores.get)
        return (best_method, results[best_method])
