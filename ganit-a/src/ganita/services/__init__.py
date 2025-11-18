"""
Shared services for mathematical discovery.

Includes evaluator, recognizer, acceleration, and cross-validation.
"""

from ganita.services.acceleration import Accelerator
from ganita.services.crossval import CrossValidator
from ganita.services.evaluator import Evaluator
from ganita.services.recognizer import Recognizer

__all__ = ["Evaluator", "Recognizer", "Accelerator", "CrossValidator"]
