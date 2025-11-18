#!/usr/bin/env python3
"""
Hacker Terminal Video Player - Main entry point
"""
import argparse
import sys
import os
from .video_player import VideoPlayer


def main():
    """Main entry point for the video player"""

    # ASCII art banner
    banner = """
\033[92m╦ ╦╔═╗╔═╗╦╔═╔═╗╦═╗  ╔╦╗╔═╗╦═╗╔╦╗╦╔╗╔╔═╗╦
╠═╣╠═╣║  ╠╩╗║╣ ╠╦╝   ║ ║╣ ╠╦╝║║║║║║║╠═╣║
╩ ╩╩ ╩╚═╝╩ ╩╚═╝╩╚═   ╩ ╚═╝╩╚═╩ ╩╩╝╚╝╩ ╩╩═╝
╦  ╦╦╔╦╗╔═╗╔═╗  ╔═╗╦  ╔═╗╦ ╦╔═╗╦═╗
╚╗╔╝║ ║║║╣ ║ ║  ╠═╝║  ╠═╣╚╦╝║╣ ╠╦╝
 ╚╝ ╩═╩╝╚═╝╚═╝  ╩  ╩═╝╩ ╩ ╩ ╚═╝╩╚═\033[0m
    """

    parser = argparse.ArgumentParser(
        description='A retro hacker-themed terminal video player',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=banner
    )

    parser.add_argument(
        'video_file',
        help='Path to the video file to play'
    )

    parser.add_argument(
        '--mode',
        choices=['pixelate', 'ascii', 'glitch', 'matrix'],
        default='pixelate',
        help='Initial render mode (default: pixelate)'
    )

    parser.add_argument(
        '--version',
        action='version',
        version='Hacker Terminal Video Player v1.0.0'
    )

    args = parser.parse_args()

    # Check if video file exists
    if not os.path.exists(args.video_file):
        print(f"\033[91mError: Video file not found: {args.video_file}\033[0m")
        sys.exit(1)

    # Print banner
    print(banner)
    print(f"\033[92mInitializing player...\033[0m\n")

    # Create and run player
    player = VideoPlayer(args.video_file, mode=args.mode)

    try:
        player.initialize()
        player.play()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        print(f"\033[91mError: {e}\033[0m")
        sys.exit(1)
    finally:
        player.cleanup()
        print("\033[92m\nPlayback ended. Terminal restored.\033[0m")


if __name__ == '__main__':
    main()
