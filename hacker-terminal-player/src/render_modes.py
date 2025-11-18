"""
Render modes for the hacker terminal video player.
Each mode transforms video frames into terminal-displayable content.
"""
import cv2
import numpy as np
from abc import ABC, abstractmethod
from typing import Tuple


class RenderMode(ABC):
    """Base class for all render modes"""

    def __init__(self):
        self.parameter = 1.0

    @abstractmethod
    def process_frame(self, frame: np.ndarray, width: int, height: int) -> np.ndarray:
        """Process a video frame and return terminal-ready output"""
        pass

    @abstractmethod
    def get_name(self) -> str:
        """Return the name of this render mode"""
        pass

    def increase_parameter(self):
        """Increase the mode's adjustable parameter"""
        self.parameter = min(self.parameter + 0.1, 2.0)

    def decrease_parameter(self):
        """Decrease the mode's adjustable parameter"""
        self.parameter = max(self.parameter - 0.1, 0.1)


class PixelatorMode(RenderMode):
    """Renders video in blocky, pixelated style"""

    def __init__(self):
        super().__init__()
        self.parameter = 1.0  # Controls pixel size multiplier

    def get_name(self) -> str:
        return f"PIXELATOR (size: {self.parameter:.1f}x)"

    def process_frame(self, frame: np.ndarray, width: int, height: int) -> np.ndarray:
        """
        Create pixelated effect by downscaling and upscaling with nearest neighbor
        """
        # Calculate downscale dimensions based on parameter
        scale_factor = max(0.1, 1.0 / (self.parameter + 1))
        small_width = max(1, int(width * scale_factor))
        small_height = max(1, int(height * scale_factor))

        # Downscale
        small_frame = cv2.resize(frame, (small_width, small_height),
                                interpolation=cv2.INTER_AREA)

        # Upscale with nearest neighbor for blocky effect
        pixelated = cv2.resize(small_frame, (width, height),
                              interpolation=cv2.INTER_NEAREST)

        return pixelated


class ASCIIMode(RenderMode):
    """Converts video frames to ASCII art"""

    # ASCII characters from darkest to lightest
    ASCII_CHARS = "@%#*+=-:. "

    def __init__(self):
        super().__init__()
        self.parameter = 1.0  # Controls contrast

    def get_name(self) -> str:
        return f"ASCII ART (contrast: {self.parameter:.1f}x)"

    def process_frame(self, frame: np.ndarray, width: int, height: int) -> np.ndarray:
        """
        Convert frame to grayscale and map brightness to ASCII characters
        Returns a frame with ASCII-like appearance
        """
        # Convert to grayscale
        gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)

        # Apply contrast adjustment
        gray = cv2.convertScaleAbs(gray, alpha=self.parameter, beta=0)

        # Resize to terminal dimensions
        resized = cv2.resize(gray, (width, height))

        # Create ASCII-mapped image
        # We'll create a visual representation by quantizing brightness levels
        num_chars = len(self.ASCII_CHARS)
        ascii_mapped = (resized // (256 // num_chars))
        ascii_mapped = np.clip(ascii_mapped, 0, num_chars - 1)

        # Convert back to brightness values for display
        brightness_mapped = (ascii_mapped * (255 // num_chars)).astype(np.uint8)

        # Convert to BGR for consistency
        result = cv2.cvtColor(brightness_mapped, cv2.COLOR_GRAY2BGR)

        # Add green tint for hacker aesthetic
        result[:, :, 0] = 0  # Remove blue channel
        result[:, :, 2] = result[:, :, 2] // 3  # Reduce red channel

        return result


class GlitchMode(RenderMode):
    """Applies random glitch effects for hacker aesthetic"""

    def __init__(self):
        super().__init__()
        self.parameter = 0.3  # Controls glitch intensity
        self.frame_counter = 0

    def get_name(self) -> str:
        return f"GLITCH (intensity: {self.parameter:.1f}x)"

    def process_frame(self, frame: np.ndarray, width: int, height: int) -> np.ndarray:
        """
        Apply random glitch effects to the frame
        """
        self.frame_counter += 1
        result = frame.copy()

        # Resize to desired dimensions
        result = cv2.resize(result, (width, height))

        # Apply glitches with probability based on parameter
        glitch_probability = min(0.5, self.parameter * 0.5)

        if np.random.random() < glitch_probability:
            # Choose random glitch effect
            glitch_type = np.random.randint(0, 5)

            if glitch_type == 0:
                # Row shifting
                shift_amount = int(width * 0.1 * self.parameter)
                num_rows = int(height * 0.3 * self.parameter)
                for _ in range(num_rows):
                    row = np.random.randint(0, height)
                    shift = np.random.randint(-shift_amount, shift_amount)
                    result[row] = np.roll(result[row], shift, axis=0)

            elif glitch_type == 1:
                # Color channel swapping
                b, g, r = cv2.split(result)
                swap_type = np.random.randint(0, 3)
                if swap_type == 0:
                    result = cv2.merge([r, g, b])
                elif swap_type == 1:
                    result = cv2.merge([b, r, g])
                else:
                    result = cv2.merge([g, b, r])

            elif glitch_type == 2:
                # Random noise
                noise = np.random.randint(0, int(50 * self.parameter),
                                         result.shape, dtype=np.uint8)
                result = cv2.add(result, noise)

            elif glitch_type == 3:
                # Horizontal tear
                tear_height = np.random.randint(0, height)
                tear_shift = int(width * 0.2 * self.parameter)
                result[tear_height:] = np.roll(result[tear_height:], tear_shift, axis=1)

            else:
                # Color channel offset
                offset = int(5 * self.parameter)
                b, g, r = cv2.split(result)
                b = np.roll(b, offset, axis=1)
                r = np.roll(r, -offset, axis=1)
                result = cv2.merge([b, g, r])

        # Add slight green tint for hacker aesthetic
        b, g, r = cv2.split(result)
        g = cv2.add(g, 20)
        result = cv2.merge([b, g, r])

        return result


class MatrixMode(RenderMode):
    """Matrix-style falling characters effect"""

    def __init__(self):
        super().__init__()
        self.parameter = 0.5  # Controls effect intensity
        self.trails = None

    def get_name(self) -> str:
        return f"MATRIX (intensity: {self.parameter:.1f}x)"

    def process_frame(self, frame: np.ndarray, width: int, height: int) -> np.ndarray:
        """
        Apply Matrix-style effect overlay
        """
        result = cv2.resize(frame, (width, height))

        # Convert to green-tinted grayscale
        gray = cv2.cvtColor(result, cv2.COLOR_BGR2GRAY)
        result = cv2.cvtColor(gray, cv2.COLOR_GRAY2BGR)

        # Apply green tint
        result[:, :, 0] = 0  # No blue
        result[:, :, 2] = result[:, :, 2] // 4  # Minimal red
        result[:, :, 1] = cv2.add(result[:, :, 1], 50)  # Enhanced green

        # Add scanline effect
        for i in range(0, height, 2):
            result[i] = cv2.subtract(result[i], 30)

        # Add random bright pixels (falling characters effect)
        num_pixels = int(width * height * 0.01 * self.parameter)
        for _ in range(num_pixels):
            x = np.random.randint(0, width)
            y = np.random.randint(0, height)
            result[y, x] = [0, 255, 50]  # Bright green

        return result


def get_all_modes():
    """Return list of all available render modes"""
    return [
        PixelatorMode(),
        ASCIIMode(),
        GlitchMode(),
        MatrixMode(),
    ]
