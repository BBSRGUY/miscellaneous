# vid-player

A creative terminal-based video player with multiple artistic rendering modes. Experience your videos in ASCII art, Matrix-style rain, color pulses, and pixelated retro blocks - all in your terminal!

## Features

🎨 **Multiple Render Modes:**
- **ASCII Art**: Classic ASCII art rendering with colored characters and multiple character sets
- **Pixelate**: Retro-style pixelated blocks with adjustable block size
- **Matrix Rain**: Falling Matrix-style characters influenced by video content
- **Color Pulse**: Ambient pulsating colors that reflect the video's color palette

🎮 **Interactive Controls:**
- Play/Pause, Seek forward/backward
- Real-time mode switching during playback
- Adjustable parameters for each rendering mode
- Progress bar and playback information

⚡ **Powered by:**
- **blessed**: Terminal UI framework
- **fluent-ffmpeg**: Video frame extraction
- **jimp**: Image processing
- Node.js async/await for smooth playback

## Prerequisites

Before running the player, you need to have FFmpeg installed on your system:

### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install ffmpeg
```

### Linux (Fedora/RHEL)
```bash
sudo dnf install ffmpeg
```

### macOS
```bash
brew install ffmpeg
```

### Windows
Download from [ffmpeg.org](https://ffmpeg.org/download.html) or use:
```bash
choco install ffmpeg
```

## Installation

1. Clone this repository:
```bash
git clone <repository-url>
cd miscellaneous
```

2. Install dependencies:
```bash
npm install
```

3. Make the CLI executable (Linux/macOS):
```bash
chmod +x src/cli.js
```

## Usage

### Basic Usage

Play a video with the default ASCII mode:
```bash
npm start <video_file>
```

Or use the direct command:
```bash
node src/cli.js <video_file>
```

### Specify Initial Render Mode

```bash
node src/cli.js video.mp4 --mode=ascii      # ASCII art mode
node src/cli.js video.mp4 --mode=pixelate   # Pixelate mode
node src/cli.js video.mp4 --mode=matrix     # Matrix rain mode
node src/cli.js video.mp4 --mode=pulse      # Color pulse mode
```

Short form:
```bash
node src/cli.js video.mp4 -m matrix
```

### Examples

```bash
# Play a video with Matrix effect
node src/cli.js ~/Videos/sample.mp4 -m matrix

# Play with pixelate effect
node src/cli.js ~/Videos/sample.mp4 -m pixelate

# Play with ASCII art (default)
node src/cli.js ~/Videos/sample.mp4
```

## Controls

While the video is playing, use these keyboard shortcuts:

| Key | Action |
|-----|--------|
| `SPACE` | Play/Pause |
| `Q` | Quit |
| `←` | Seek backward 5 seconds |
| `→` | Seek forward 5 seconds |
| `M` | Cycle through render modes |
| `+` or `=` | Increase mode parameter |
| `-` | Decrease mode parameter |

## Render Modes Explained

### ASCII Art Mode
Converts video frames to ASCII characters based on brightness and color. The parameter controls character density (more characters = more detail).

**Parameter Effect:** Adjusts the range of ASCII characters used (more = finer detail)

### Pixelate Mode
Creates a retro pixelated effect by averaging colors into blocks.

**Parameter Effect:** Controls block size (lower = larger pixels, more pixelated)

### Matrix Rain Mode
Inspired by "The Matrix," creates falling streams of characters influenced by the video's content and colors.

**Parameter Effect:** Adjusts the trail length of falling characters

### Color Pulse Mode
Doesn't show the video directly but creates a mesmerizing pulsating pattern based on the average color of each frame.

**Parameter Effect:** Controls pulse speed and animation

## Project Structure

```
miscellaneous/
├── package.json
├── README.md
└── src/
    ├── cli.js              # CLI entry point
    ├── App.js              # Main application controller
    ├── VideoPlayer.js      # Video processing and playback
    ├── TerminalUI.js       # Terminal interface wrapper
    └── renderers/
        ├── BaseRenderer.js          # Base renderer class
        ├── ASCIIRenderer.js        # ASCII art renderer
        ├── PixelateRenderer.js     # Pixelate renderer
        ├── MatrixRenderer.js       # Matrix rain renderer
        └── ColorPulseRenderer.js   # Color pulse renderer
```

## How It Works

1. **Video Processing**: The player uses FFmpeg (via fluent-ffmpeg) to extract individual frames from the video file
2. **Image Analysis**: Each frame is processed by Jimp to analyze colors, brightness, and patterns
3. **Terminal Rendering**: The blessed library creates a rich terminal UI with the processed frames
4. **Render Modes**: Different rendering algorithms transform the frame data into various artistic representations
5. **Playback Loop**: Frames are rendered at the video's framerate (capped at 30 FPS) for smooth playback

## Performance Notes

- Frame extraction happens on-demand and frames are cached in a temporary directory
- The player targets 30 FPS maximum to ensure smooth terminal rendering
- Larger terminal windows may require more processing power
- Some render modes (Matrix, Color Pulse) are more CPU-intensive than others

## Limitations

- Audio is not currently supported (video frames only)
- Performance depends on terminal size and video resolution
- Very high-resolution videos may need downscaling for best performance
- The player creates temporary frame files that are cleaned up on exit

## Troubleshooting

### "FFmpeg not found" error
Make sure FFmpeg is installed and available in your PATH:
```bash
ffmpeg -version
```

### Slow playback
- Try reducing your terminal window size
- Use a lower resolution video
- Switch to a less CPU-intensive mode (ASCII or Pixelate)

### Colors not displaying correctly
- Ensure your terminal supports 24-bit true color
- Try a different terminal emulator (iTerm2, Alacritty, or modern terminals)

## Future Enhancements

Potential features for future versions:
- Audio playback synchronization
- Edge detection / Vector Scan mode
- Glitch effect mode
- Recording terminal output to file
- Network streaming support
- Playlist support

## License

MIT

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

---

**Enjoy your terminal video experience!** 🎬✨
