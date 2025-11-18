"""
PSLQ/LLL-based constant recognizer with symbolic simplification.

Recognizes numeric values as integer relations involving known constants.
"""

from __future__ import annotations

import logging
from typing import Any

import mpmath as mp
import sympy as sp
from sympy import E, catalan, euler_gamma, log, pi, zoo

from ganita.models import RecognizerSummary

logger = logging.getLogger(__name__)


class ConstantBasis:
    """Known constants basis for recognition."""

    BASES: dict[str, dict[str, Any]] = {
        "small_v1": {
            "symbols": ["1", "PI", "LOG2", "ZETA3", "CATALAN", "EULER_GAMMA"],
            "max_height": 2000,
        },
        "medium_v1": {
            "symbols": [
                "1",
                "PI",
                "PI2",
                "LOG2",
                "LOG3",
                "LOG5",
                "ZETA3",
                "CATALAN",
                "EULER_GAMMA",
                "SQRT2",
                "SQRT3",
                "SQRT5",
                "PHI",
            ],
            "max_height": 10000,
        },
        "extended_v1": {
            "symbols": [
                "1",
                "PI",
                "PI2",
                "PI3",
                "E",
                "LOG2",
                "LOG3",
                "LOG5",
                "LOG10",
                "ZETA3",
                "ZETA5",
                "CATALAN",
                "EULER_GAMMA",
                "SQRT2",
                "SQRT3",
                "SQRT5",
                "PHI",
                "LI2_HALF",
            ],
            "max_height": 50000,
        },
    }

    @staticmethod
    def get_basis_values(basis_name: str, precision: int) -> list[mp.mpf]:
        """Get numeric values for a basis at given precision."""
        mp.dps = precision

        # Define constant computations
        constants_map = {
            "1": mp.mpf(1),
            "PI": mp.pi,
            "PI2": mp.pi**2,
            "PI3": mp.pi**3,
            "E": mp.e,
            "LOG2": mp.log(2),
            "LOG3": mp.log(3),
            "LOG5": mp.log(5),
            "LOG10": mp.log(10),
            "ZETA3": mp.zeta(3),
            "ZETA5": mp.zeta(5),
            "CATALAN": mp.catalan,
            "EULER_GAMMA": mp.euler,
            "SQRT2": mp.sqrt(2),
            "SQRT3": mp.sqrt(3),
            "SQRT5": mp.sqrt(5),
            "PHI": (1 + mp.sqrt(5)) / 2,
            "LI2_HALF": mp.polylog(2, 0.5),
        }

        basis_info = ConstantBasis.BASES.get(basis_name, ConstantBasis.BASES["small_v1"])
        symbols = basis_info["symbols"]

        values = []
        for sym in symbols:
            if sym in constants_map:
                values.append(constants_map[sym])
            else:
                logger.warning(f"Unknown constant symbol: {sym}")
                values.append(mp.mpf(0))

        return values

    @staticmethod
    def get_basis_symbols(basis_name: str) -> list[str]:
        """Get symbolic names for a basis."""
        basis_info = ConstantBasis.BASES.get(basis_name, ConstantBasis.BASES["small_v1"])
        return basis_info["symbols"]


class Recognizer:
    """PSLQ-based constant recognizer."""

    def __init__(self, config: dict[str, Any]):
        self.config = config
        self.bases_config = config.get("bases", [ConstantBasis.BASES["small_v1"]])
        self.precision_levels = config.get("pslq_precision_digits", [200, 400, 800])

    def recognize(self, value: str, stage_check: bool = True) -> RecognizerSummary | None:
        """
        Attempt to recognize a numeric value against all configured bases.

        Returns the best recognition result or None if no match found.
        """
        value_mpf = mp.mpf(value)
        best_result = None
        best_height = float("inf")

        for basis_config in self.bases_config:
            basis_name = basis_config.get("name", "unnamed")
            max_height = basis_config.get("max_height", 2000)

            for precision in self.precision_levels:
                result = self._try_basis(
                    value_mpf, basis_name, precision, max_height, stage_check
                )

                if result and result.success:
                    # Keep result with smallest coefficient height
                    if result.height is not None and result.height < best_height:
                        best_result = result
                        best_height = result.height

                    # If we found a very simple relation, stop
                    if result.height and result.height < 100:
                        return result

        return best_result

    def _try_basis(
        self,
        value: mp.mpf,
        basis_name: str,
        precision: int,
        max_height: int,
        stage_check: bool,
    ) -> RecognizerSummary | None:
        """Try PSLQ recognition against a specific basis and precision."""
        mp.dps = precision + 50  # Extra precision for PSLQ

        try:
            # Get basis values
            basis_values = ConstantBasis.get_basis_values(basis_name, precision)
            basis_symbols = ConstantBasis.get_basis_symbols(basis_name)

            # Build input vector: [value, basis_values...]
            input_vector = [value] + basis_values

            # Run PSLQ
            relation = mp.pslq(input_vector, maxcoeff=max_height, maxsteps=10000)

            if relation is None:
                return RecognizerSummary(
                    basis_name=basis_name,
                    success=False,
                    precision_used=precision,
                )

            # Extract coefficients
            coeffs = [int(c) for c in relation]

            # Verify relation quality
            residual = sum(c * v for c, v in zip(coeffs, input_vector))
            residual_log10 = float(mp.log10(abs(residual))) if residual != 0 else -999

            # Check if relation is meaningful (first coeff should be non-zero)
            if coeffs[0] == 0:
                return RecognizerSummary(
                    basis_name=basis_name,
                    success=False,
                    precision_used=precision,
                )

            # Compute height
            height = max(abs(c) for c in coeffs)

            # Build human-readable relation
            relation_tex = self._build_tex_relation(coeffs, basis_symbols)
            relation_json = {
                "coefficients": coeffs,
                "symbols": ["VALUE"] + basis_symbols,
            }

            # Stage check: verify at higher precision if requested
            stability_verified = False
            if stage_check and len(self.precision_levels) > 1:
                # Try at next precision level
                next_precision = precision * 2
                mp.dps = next_precision + 50
                basis_values_2 = ConstantBasis.get_basis_values(basis_name, next_precision)
                input_vector_2 = [value] + basis_values_2
                residual_2 = sum(c * v for c, v in zip(coeffs, input_vector_2))
                residual_log10_2 = (
                    float(mp.log10(abs(residual_2))) if residual_2 != 0 else -999
                )

                # Relation is stable if residual improves with precision
                stability_verified = residual_log10_2 < residual_log10 - 10

            return RecognizerSummary(
                basis_name=basis_name,
                success=True,
                relation_tex=relation_tex,
                relation_json=relation_json,
                height=height,
                residual_log10=residual_log10,
                precision_used=precision,
                stability_verified=stability_verified,
            )

        except Exception as e:
            logger.debug(f"PSLQ failed for basis {basis_name} at precision {precision}: {e}")
            return RecognizerSummary(
                basis_name=basis_name,
                success=False,
                precision_used=precision,
            )

    def _build_tex_relation(self, coeffs: list[int], symbols: list[str]) -> str:
        """Build LaTeX representation of the relation."""
        # coeffs[0] * VALUE + coeffs[1] * symbols[0] + ... = 0
        # Rearrange to: VALUE = -1/coeffs[0] * (coeffs[1] * symbols[0] + ...)

        if coeffs[0] == 0:
            return "Invalid relation (first coefficient is zero)"

        terms = []
        for i, (c, sym) in enumerate(zip(coeffs[1:], symbols), 1):
            if c == 0:
                continue

            # Format coefficient
            coeff_str = ""
            if c == 1:
                coeff_str = ""
            elif c == -1:
                coeff_str = "-"
            else:
                coeff_str = str(c)

            # Format symbol
            sym_tex = self._symbol_to_tex(sym)

            # Build term
            if coeff_str == "-":
                term = f"- {sym_tex}"
            elif coeff_str:
                term = f"{coeff_str} {sym_tex}"
            else:
                term = sym_tex

            terms.append((c > 0, term))

        if not terms:
            return "VALUE = 0"

        # Build RHS
        rhs_parts = []
        for i, (is_positive, term) in enumerate(terms):
            if i == 0:
                if is_positive:
                    rhs_parts.append(term)
                else:
                    rhs_parts.append(term)
            else:
                if is_positive:
                    rhs_parts.append(f"+ {term}")
                else:
                    rhs_parts.append(term)

        rhs = " ".join(rhs_parts)

        # Apply coefficient from VALUE
        if coeffs[0] == 1:
            return f"VALUE = -({rhs})"
        elif coeffs[0] == -1:
            return f"VALUE = {rhs}"
        else:
            return f"VALUE = \\frac{{-1}}{{{coeffs[0]}}} ({rhs})"

    def _symbol_to_tex(self, symbol: str) -> str:
        """Convert symbol name to LaTeX."""
        tex_map = {
            "1": "1",
            "PI": "\\pi",
            "PI2": "\\pi^2",
            "PI3": "\\pi^3",
            "E": "e",
            "LOG2": "\\log 2",
            "LOG3": "\\log 3",
            "LOG5": "\\log 5",
            "LOG10": "\\log 10",
            "ZETA3": "\\zeta(3)",
            "ZETA5": "\\zeta(5)",
            "CATALAN": "G",
            "EULER_GAMMA": "\\gamma",
            "SQRT2": "\\sqrt{2}",
            "SQRT3": "\\sqrt{3}",
            "SQRT5": "\\sqrt{5}",
            "PHI": "\\phi",
            "LI2_HALF": "\\text{Li}_2(1/2)",
        }
        return tex_map.get(symbol, symbol)

    def simplify_symbolic(self, relation_json: dict[str, Any]) -> str | None:
        """
        Use SymPy to symbolically simplify the relation.

        Returns simplified expression or None if simplification fails.
        """
        try:
            coeffs = relation_json["coefficients"]
            symbols_list = relation_json["symbols"]

            # Build SymPy expression
            expr = sp.Integer(0)

            for c, sym in zip(coeffs[1:], symbols_list[1:]):
                if c == 0:
                    continue

                sym_val = self._symbol_to_sympy(sym)
                expr += sp.Integer(c) * sym_val

            # Divide by coefficient of VALUE
            if coeffs[0] != 0:
                expr = -expr / sp.Integer(coeffs[0])

            # Simplify
            simplified = sp.simplify(expr)

            return str(simplified)

        except Exception as e:
            logger.debug(f"Symbolic simplification failed: {e}")
            return None

    def _symbol_to_sympy(self, symbol: str) -> sp.Basic:
        """Convert symbol name to SymPy expression."""
        sympy_map = {
            "1": sp.Integer(1),
            "PI": sp.pi,
            "PI2": sp.pi**2,
            "PI3": sp.pi**3,
            "E": sp.E,
            "LOG2": sp.log(2),
            "LOG3": sp.log(3),
            "LOG5": sp.log(5),
            "LOG10": sp.log(10),
            "ZETA3": sp.zeta(3),
            "ZETA5": sp.zeta(5),
            "CATALAN": sp.catalan,
            "EULER_GAMMA": sp.EulerGamma,
            "SQRT2": sp.sqrt(2),
            "SQRT3": sp.sqrt(3),
            "SQRT5": sp.sqrt(5),
            "PHI": (1 + sp.sqrt(5)) / 2,
            "LI2_HALF": sp.polylog(2, sp.Rational(1, 2)),
        }
        return sympy_map.get(symbol, sp.Symbol(symbol))
