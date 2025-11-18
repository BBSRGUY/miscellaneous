"""
Terminal rendering engine for the hacker video player.
Handles displaying frames in the terminal using block characters and ANSI colors.
"""
import sys
import numpy as np
from blessed import Terminal


class TerminalRenderer:
    """Renders video frames to the terminal using colored block characters"""

    def __init__(self):
        self.term = Terminal()
        self.block_char = '█'
        self.half_block = '▄'

    def get_dimensions(self):
        """Get usable terminal dimensions (width, height)"""
        # Reserve bottom 3 lines for controls/info
        return self.term.width, max(1, self.term.height - 4)

    def clear_screen(self):
        """Clear the terminal screen"""
        print(self.term.home + self.term.clear, end='')

    def rgb_to_ansi(self, r, g, b):
        """Convert RGB to ANSI 256 color code"""
        # Use 256-color palette
        # Map RGB (0-255) to 6x6x6 color cube (16-231) or grayscale (232-255)

        # Check if it's grayscale
        if abs(r - g) < 10 and abs(g - b) < 10 and abs(r - b) < 10:
            # Use grayscale ramp (232-255)
            gray = int((r + g + b) / 3)
            if gray < 8:
                return 16  # Black
            elif gray > 247:
                return 231  # White
            else:
                return 232 + int((gray - 8) / 10)

        # Use 6x6x6 color cube
        r_idx = int(r / 51)  # 0-5
        g_idx = int(g / 51)  # 0-5
        b_idx = int(b / 51)  # 0-5

        return 16 + (36 * r_idx) + (6 * g_idx) + b_idx

    def render_frame(self, frame: np.ndarray):
        """
        Render a frame to the terminal using colored blocks.
        Frame should be BGR format (OpenCV default).
        """
        if frame is None or frame.size == 0:
            return

        height, width = frame.shape[:2]

        # Move cursor to home position
        output = self.term.home

        # Render frame line by line using half-blocks for better vertical resolution
        for y in range(0, height - 1, 2):
            line = ''
            for x in range(width):
                # Get top and bottom pixel colors
                b_top, g_top, r_top = frame[y, x]
                b_bot, g_bot, r_bot = frame[y + 1, x]

                # Convert BGR to RGB and then to ANSI color
                fg_color = self.rgb_to_ansi(int(r_top), int(g_top), int(b_top))
                bg_color = self.rgb_to_ansi(int(r_bot), int(g_bot), int(b_bot))

                # Use half-block with different foreground/background colors
                line += f'\033[38;5;{fg_color}m\033[48;5;{bg_color}m{self.half_block}'

            # Reset colors at end of line and add newline
            output += line + '\033[0m\n'

        # Write the entire frame at once for better performance
        sys.stdout.write(output)
        sys.stdout.flush()

    def render_controls(self, status_info: dict):
        """
        Render control information at the bottom of the screen
        """
        width = self.term.width

        # Move to bottom of screen
        with self.term.location(0, self.term.height - 3):
            # Create separator line
            separator = '─' * width
            print(self.term.green(separator))

            # Status line
            state = "▶ PLAYING" if status_info.get('playing', False) else "⏸ PAUSED"
            mode = status_info.get('mode', 'UNKNOWN')
            frame_info = f"Frame: {status_info.get('current_frame', 0)}/{status_info.get('total_frames', 0)}"
            time_info = f"Time: {status_info.get('current_time', '0:00')}/{status_info.get('total_time', '0:00')}"

            status_line = f"{state} | Mode: {mode} | {frame_info} | {time_info}"
            print(self.term.green(status_line[:width]))

            # Controls line
            controls = "[SPACE] Play/Pause  [M] Mode  [+/-] Adjust  [←/→] Seek  [Q] Quit"
            print(self.term.bright_green(controls[:width]))

    def show_splash(self, video_path: str):
        """Show startup splash screen"""
        self.clear_screen()

        splash = f"""
{self.term.bright_green}
    ╦ ╦╔═╗╔═╗╦╔═╔═╗╦═╗  ╔╦╗╔═╗╦═╗╔╦╗╦╔╗╔╔═╗╦
    ╠═╣╠═╣║  ╠╩╗║╣ ╠╦╝   ║ ║╣ ╠╦╝║║║║║║║╠═╣║
    ╩ ╩╩ ╩╚═╝╩ ╩╚═╝╩╚═   ╩ ╚═╝╩╚═╩ ╩╩╝╚╝╩ ╩╩═╝
    ╦  ╦╦╔╦╗╔═╗╔═╗  ╔═╗╦  ╔═╗╦ ╦╔═╗╦═╗
    ╚╗╔╝║ ║║║╣ ║ ║  ╠═╝║  ╠═╣╚╦╝║╣ ╠╦╝
     ╚╝ ╩═╩╝╚═╝╚═╝  ╩  ╩═╝╩ ╩ ╩ ╚═╝╩╚═
{self.term.normal}

{self.term.green}Loading: {video_path}{self.term.normal}

{self.term.bright_black}Initializing terminal rendering engine...{self.term.normal}
"""
        print(splash)
        sys.stdout.flush()

    def cleanup(self):
        """Cleanup terminal state"""
        print(self.term.normal + self.term.clear)
        sys.stdout.flush()
