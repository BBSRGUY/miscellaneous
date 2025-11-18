#!/usr/bin/env python3
"""
Create a demo video for testing the Hacker Terminal Video Player
Generates a simple animated video with colorful patterns
"""
import cv2
import numpy as np
import os


def create_demo_video(output_path='examples/demo.mp4', duration=10, fps=30):
    """Create a demo video with animated patterns"""

    # Video properties
    width, height = 640, 480
    total_frames = duration * fps

    # Create output directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    # Initialize video writer
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_path, fourcc, fps, (width, height))

    print(f"Generating demo video: {output_path}")
    print(f"Duration: {duration}s, FPS: {fps}, Resolution: {width}x{height}")

    for frame_num in range(total_frames):
        # Create a frame
        frame = np.zeros((height, width, 3), dtype=np.uint8)

        # Progress (0 to 1)
        progress = frame_num / total_frames

        # Pattern 1: Moving gradient
        for y in range(height):
            for x in range(width):
                r = int(128 + 127 * np.sin((x + frame_num * 2) * 0.02))
                g = int(128 + 127 * np.sin((y + frame_num * 2) * 0.02))
                b = int(128 + 127 * np.sin((x + y + frame_num * 2) * 0.01))
                frame[y, x] = [b, g, r]

        # Pattern 2: Moving circles
        num_circles = 5
        for i in range(num_circles):
            angle = (progress * 2 * np.pi) + (i * 2 * np.pi / num_circles)
            cx = int(width / 2 + width / 3 * np.cos(angle))
            cy = int(height / 2 + height / 3 * np.sin(angle))
            color = (
                int(255 * (i / num_circles)),
                int(255 * (1 - i / num_circles)),
                128
            )
            cv2.circle(frame, (cx, cy), 30, color, -1)

        # Add text
        text = f"HACKER TERMINAL PLAYER - DEMO"
        font = cv2.FONT_HERSHEY_BOLD
        text_size = cv2.getTextSize(text, font, 1, 2)[0]
        text_x = (width - text_size[0]) // 2
        text_y = 50

        # Text shadow
        cv2.putText(frame, text, (text_x + 2, text_y + 2), font, 1, (0, 0, 0), 2)
        # Text
        cv2.putText(frame, text, (text_x, text_y), font, 1, (0, 255, 0), 2)

        # Frame counter
        counter_text = f"Frame: {frame_num + 1}/{total_frames}"
        cv2.putText(frame, counter_text, (10, height - 20),
                   cv2.FONT_HERSHEY_SIMPLEX, 0.6, (255, 255, 255), 1)

        # Write frame
        out.write(frame)

        # Progress indicator
        if (frame_num + 1) % fps == 0:
            print(f"  Generated {frame_num + 1}/{total_frames} frames "
                  f"({int((frame_num + 1) / total_frames * 100)}%)")

    # Release video writer
    out.release()
    print(f"\n✓ Demo video created: {output_path}")
    print(f"\nTo play the demo video, run:")
    print(f"  ./vid-player {output_path}")


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(description='Create a demo video for testing')
    parser.add_argument('--output', default='examples/demo.mp4',
                       help='Output video path (default: examples/demo.mp4)')
    parser.add_argument('--duration', type=int, default=10,
                       help='Duration in seconds (default: 10)')
    parser.add_argument('--fps', type=int, default=30,
                       help='Frames per second (default: 30)')

    args = parser.parse_args()

    create_demo_video(args.output, args.duration, args.fps)
