"""
Core video player class that integrates video processing, rendering, and controls.
"""
import cv2
import time
import threading
import os
import sys
from .terminal_renderer import TerminalRenderer
from .render_modes import get_all_modes


class VideoPlayer:
    """Main video player class"""

    def __init__(self, video_path: str, mode: str = "pixelate"):
        self.video_path = video_path
        self.cap = None
        self.renderer = TerminalRenderer()
        self.modes = get_all_modes()
        self.current_mode_idx = 0

        # Set initial mode based on parameter
        mode_map = {
            'pixelate': 0,
            'ascii': 1,
            'glitch': 2,
            'matrix': 3
        }
        self.current_mode_idx = mode_map.get(mode.lower(), 0)

        # Playback state
        self.playing = False
        self.quit_flag = False
        self.current_frame_num = 0
        self.total_frames = 0
        self.fps = 30.0
        self.frame_delay = 1.0 / self.fps

        # Audio thread
        self.audio_thread = None
        self.audio_initialized = False

        # Input handling
        self.input_thread = None

    def initialize(self):
        """Initialize video capture and audio"""
        # Open video file
        self.cap = cv2.VideoCapture(self.video_path)

        if not self.cap.isOpened():
            raise RuntimeError(f"Failed to open video file: {self.video_path}")

        # Get video properties
        self.fps = self.cap.get(cv2.CAP_PROP_FPS)
        if self.fps == 0:
            self.fps = 30.0
        self.frame_delay = 1.0 / self.fps
        self.total_frames = int(self.cap.get(cv2.CAP_PROP_FRAME_COUNT))

        # Show splash screen
        self.renderer.show_splash(self.video_path)
        time.sleep(1.5)

        # Initialize audio in separate thread
        self.audio_thread = threading.Thread(target=self._init_audio, daemon=True)
        self.audio_thread.start()

        return True

    def _init_audio(self):
        """Initialize pygame audio and start playback"""
        try:
            import pygame
            pygame.mixer.init()

            # Load and play audio if video has it
            if os.path.exists(self.video_path):
                # Extract audio to temp file (simplified - in production use proper temp file)
                # For now, just try to play the video file directly
                try:
                    pygame.mixer.music.load(self.video_path)
                    self.audio_initialized = True
                except:
                    # Video might not have audio or pygame can't handle the format
                    self.audio_initialized = False
        except ImportError:
            self.audio_initialized = False

    def _play_audio(self):
        """Start audio playback"""
        if self.audio_initialized:
            try:
                import pygame
                pygame.mixer.music.play()
            except:
                pass

    def _pause_audio(self):
        """Pause audio playback"""
        if self.audio_initialized:
            try:
                import pygame
                pygame.mixer.music.pause()
            except:
                pass

    def _resume_audio(self):
        """Resume audio playback"""
        if self.audio_initialized:
            try:
                import pygame
                pygame.mixer.music.unpause()
            except:
                pass

    def _seek_audio(self, position_sec: float):
        """Seek audio to position"""
        if self.audio_initialized:
            try:
                import pygame
                pygame.mixer.music.play(start=position_sec)
            except:
                pass

    def play(self):
        """Start main playback loop"""
        self.playing = True

        # Start input handling thread
        self.input_thread = threading.Thread(target=self._handle_input, daemon=True)
        self.input_thread.start()

        # Start audio
        self._play_audio()

        # Clear screen
        self.renderer.clear_screen()

        # Main playback loop
        last_frame_time = time.time()

        while not self.quit_flag:
            if self.playing:
                # Read frame
                ret, frame = self.cap.read()

                if not ret:
                    # End of video - loop back
                    self.cap.set(cv2.CAP_PROP_POS_FRAMES, 0)
                    self.current_frame_num = 0
                    self._seek_audio(0)
                    continue

                self.current_frame_num = int(self.cap.get(cv2.CAP_PROP_POS_FRAMES))

                # Get terminal dimensions
                term_width, term_height = self.renderer.get_dimensions()

                # Process frame with current mode
                current_mode = self.modes[self.current_mode_idx]
                processed_frame = current_mode.process_frame(frame, term_width, term_height * 2)

                # Render frame to terminal
                self.renderer.render_frame(processed_frame)

                # Render controls
                status_info = {
                    'playing': self.playing,
                    'mode': current_mode.get_name(),
                    'current_frame': self.current_frame_num,
                    'total_frames': self.total_frames,
                    'current_time': self._format_time(self.current_frame_num / self.fps),
                    'total_time': self._format_time(self.total_frames / self.fps),
                }
                self.renderer.render_controls(status_info)

                # Frame timing
                elapsed = time.time() - last_frame_time
                sleep_time = max(0, self.frame_delay - elapsed)
                time.sleep(sleep_time)
                last_frame_time = time.time()

            else:
                # Paused - just render controls
                current_mode = self.modes[self.current_mode_idx]
                status_info = {
                    'playing': self.playing,
                    'mode': current_mode.get_name(),
                    'current_frame': self.current_frame_num,
                    'total_frames': self.total_frames,
                    'current_time': self._format_time(self.current_frame_num / self.fps),
                    'total_time': self._format_time(self.total_frames / self.fps),
                }
                self.renderer.render_controls(status_info)
                time.sleep(0.1)

    def _handle_input(self):
        """Handle keyboard input in separate thread"""
        import termios
        import tty

        # Save terminal settings
        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)

        try:
            tty.setcbreak(fd)

            while not self.quit_flag:
                # Check if input is available
                import select
                if select.select([sys.stdin], [], [], 0.1)[0]:
                    char = sys.stdin.read(1)

                    if char == ' ':
                        # Toggle play/pause
                        self.playing = not self.playing
                        if self.playing:
                            self._resume_audio()
                        else:
                            self._pause_audio()

                    elif char == 'q' or char == 'Q':
                        # Quit
                        self.quit_flag = True

                    elif char == 'm' or char == 'M':
                        # Cycle through modes
                        self.current_mode_idx = (self.current_mode_idx + 1) % len(self.modes)

                    elif char == '+' or char == '=':
                        # Increase parameter
                        self.modes[self.current_mode_idx].increase_parameter()

                    elif char == '-' or char == '_':
                        # Decrease parameter
                        self.modes[self.current_mode_idx].decrease_parameter()

                    elif char == '\x1b':  # ESC sequence
                        # Read next two characters for arrow keys
                        next_chars = sys.stdin.read(2)
                        if next_chars == '[C':  # Right arrow
                            self._seek_forward()
                        elif next_chars == '[D':  # Left arrow
                            self._seek_backward()

        finally:
            # Restore terminal settings
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)

    def _seek_forward(self):
        """Seek forward 5 seconds"""
        target_frame = min(self.current_frame_num + int(5 * self.fps), self.total_frames - 1)
        self.cap.set(cv2.CAP_PROP_POS_FRAMES, target_frame)
        self.current_frame_num = target_frame
        self._seek_audio(target_frame / self.fps)

    def _seek_backward(self):
        """Seek backward 5 seconds"""
        target_frame = max(self.current_frame_num - int(5 * self.fps), 0)
        self.cap.set(cv2.CAP_PROP_POS_FRAMES, target_frame)
        self.current_frame_num = target_frame
        self._seek_audio(target_frame / self.fps)

    def _format_time(self, seconds: float) -> str:
        """Format seconds as MM:SS"""
        mins = int(seconds // 60)
        secs = int(seconds % 60)
        return f"{mins}:{secs:02d}"

    def cleanup(self):
        """Clean up resources"""
        if self.cap:
            self.cap.release()

        self.renderer.cleanup()

        # Stop audio
        if self.audio_initialized:
            try:
                import pygame
                pygame.mixer.music.stop()
                pygame.mixer.quit()
            except:
                pass
