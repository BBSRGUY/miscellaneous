"""
Ganit-A: Advanced Mathematical Discovery Engine

Automatically discover, evaluate, recognize, and validate mathematical constants
and patterns using rigorous interval arithmetic and cross-validation.
"""

__version__ = "1.0.0"
__author__ = "Ganit-A Team"

from ganita.models import Discovery, Evidence, IntervalSample, ConfidenceStage

__all__ = [
    "Discovery",
    "Evidence",
    "IntervalSample",
    "ConfidenceStage",
]
