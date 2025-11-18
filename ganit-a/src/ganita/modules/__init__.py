"""
Discovery modules for Ganit-A.

Each module explores a specific class of mathematical expressions.
"""

from ganita.modules.base import BaseModule
from ganita.modules.continued_fractions import ContinuedFractionsModule
from ganita.modules.fixed_points import FixedPointsModule
from ganita.modules.series_products import SeriesProductsModule

__all__ = [
    "BaseModule",
    "SeriesProductsModule",
    "ContinuedFractionsModule",
    "FixedPointsModule",
]
