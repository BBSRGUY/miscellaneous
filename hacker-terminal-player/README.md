# 🎬 Hacker Terminal Video Player

A retro, hacker-themed terminal-based video player with multiple visual effects and real-time rendering modes. Experience video playback like you're in a cyberpunk movie!

```
╦ ╦╔═╗╔═╗╦╔═╔═╗╦═╗  ╔╦╗╔═╗╦═╗╔╦╗╦╔╗╔╔═╗╦
╠═╣╠═╣║  ╠╩╗║╣ ╠╦╝   ║ ║╣ ╠╦╝║║║║║║║╠═╣║
╩ ╩╩ ╩╚═╝╩ ╩╚═╝╩╚═   ╩ ╚═╝╩╚═╩ ╩╩╝╚╝╩ ╩╩═╝
╦  ╦╦╔╦╗╔═╗╔═╗  ╔═╗╦  ╔═╗╦ ╦╔═╗╦═╗
╚╗╔╝║ ║║║╣ ║ ║  ╠═╝║  ╠═╣╚╦╝║╣ ╠╦╝
 ╚╝ ╩═╩╝╚═╝╚═╝  ╩  ╩═╝╩ ╩ ╩ ╚═╝╩╚═
```

## ✨ Features

- **Multiple Render Modes**: Switch between different visual effects in real-time
  - **Pixelator**: Blocky, pixelated retro style
  - **ASCII Art**: Classic ASCII character conversion
  - **Glitch**: Random visual artifacts and distortions
  - **Matrix**: Falling characters effect with green tint

- **Real-time Control**: Adjust visual parameters on the fly
- **Full Playback Controls**: Play, pause, seek, and loop
- **Audio Synchronization**: Audio playback synced with video (when available)
- **Hacker Aesthetic**: Green-on-black terminal interface with retro styling

## 🚀 Installation

### Prerequisites

- Python 3.8 or higher
- A terminal with 256-color support
- FFmpeg (for audio support)

### Install Dependencies

```bash
cd hacker-terminal-player
pip install -r requirements.txt
```

Or install manually:
```bash
pip install opencv-python numpy pygame blessed
```

## 📖 Usage

### Basic Usage

```bash
./vid-player path/to/video.mp4
```

### With Specific Render Mode

```bash
./vid-player path/to/video.mp4 --mode=glitch
```

### Available Modes

- `pixelate` - Pixelated blocky style (default)
- `ascii` - ASCII art conversion
- `glitch` - Glitch effects
- `matrix` - Matrix-style falling characters

### Python Module Usage

You can also run it as a Python module:

```bash
python -m src.main path/to/video.mp4 --mode=matrix
```

## 🎮 Controls

During playback, use the following keyboard shortcuts:

| Key | Action |
|-----|--------|
| `SPACE` | Play/Pause |
| `M` | Cycle through render modes |
| `+` / `=` | Increase effect parameter |
| `-` / `_` | Decrease effect parameter |
| `←` | Seek backward 5 seconds |
| `→` | Seek forward 5 seconds |
| `Q` | Quit player |

## 🎨 Render Modes Explained

### Pixelator Mode
Creates a blocky, pixelated effect reminiscent of retro video games. Adjust the pixel size with `+` and `-` keys.

**Parameter**: Pixel size multiplier (0.1x - 2.0x)

### ASCII Art Mode
Converts each frame to ASCII-style art using brightness mapping. Adjust contrast for different effects.

**Parameter**: Contrast multiplier (0.1x - 2.0x)

### Glitch Mode
Applies random visual artifacts including:
- Row/column shifting
- Color channel swapping
- Random noise
- Horizontal tears
- Color channel offsets

**Parameter**: Glitch intensity (0.1x - 2.0x)

### Matrix Mode
Inspired by "The Matrix," creates a green-tinted display with falling characters effect and scanlines.

**Parameter**: Effect intensity (0.1x - 2.0x)

## 🛠️ Technical Details

### Architecture

The player consists of three main components:

1. **Video Processing Backend** (`src/video_player.py`)
   - Uses OpenCV for video decoding and frame extraction
   - Handles playback timing and synchronization

2. **Render Modes** (`src/render_modes.py`)
   - Modular render mode system
   - Each mode implements frame processing independently
   - Easy to add new visual effects

3. **Terminal Renderer** (`src/terminal_renderer.py`)
   - Uses `blessed` library for terminal control
   - Renders frames using ANSI 256-color palette
   - Uses half-block characters for double vertical resolution

### Performance Considerations

- Terminal size affects performance (smaller terminals = faster rendering)
- Some render modes (especially Glitch) may be CPU-intensive
- For best results, use a terminal with hardware acceleration
- Recommended terminal size: 80x24 to 160x48

## 🖥️ Tested Terminals

Works best with:
- iTerm2 (macOS)
- Terminal.app (macOS)
- GNOME Terminal (Linux)
- Konsole (Linux)
- Windows Terminal (Windows)

## 📝 Examples

Play a video with pixelated effect:
```bash
./vid-player examples/sample.mp4 --mode=pixelate
```

Start with glitch mode for that authentic hacker feel:
```bash
./vid-player ~/Videos/matrix.mp4 --mode=glitch
```

## 🐛 Troubleshooting

### Colors look wrong
Make sure your terminal supports 256 colors. Test with:
```bash
echo $TERM
```
Should output something like `xterm-256color`.

### Audio not playing
- Make sure pygame is installed: `pip install pygame`
- Check that your video file has an audio track
- Try converting your video with: `ffmpeg -i input.mp4 -c:a libvorbis output.mp4`

### Player is laggy
- Try reducing your terminal window size
- Use a simpler render mode (ASCII is usually fastest)
- Reduce the effect parameter with `-` key

### Video not found
Make sure to provide the full path to your video file or use a relative path from your current directory.

## 🔧 Development

### Project Structure

```
hacker-terminal-player/
├── src/
│   ├── __init__.py
│   ├── main.py              # CLI entry point
│   ├── video_player.py      # Core player logic
│   ├── terminal_renderer.py # Terminal rendering engine
│   └── render_modes.py      # Visual effect implementations
├── tests/                   # Unit tests (coming soon)
├── examples/                # Example videos
├── requirements.txt         # Python dependencies
├── setup.py                 # Package setup
├── vid-player              # Launcher script
└── README.md               # This file
```

### Adding New Render Modes

1. Create a new class in `src/render_modes.py` that inherits from `RenderMode`
2. Implement `process_frame()` and `get_name()` methods
3. Add your mode to the `get_all_modes()` function
4. Update the mode map in `video_player.py`

Example:
```python
class MyCustomMode(RenderMode):
    def get_name(self) -> str:
        return f"MY MODE (param: {self.parameter:.1f}x)"

    def process_frame(self, frame: np.ndarray, width: int, height: int) -> np.ndarray:
        # Your custom processing here
        result = cv2.resize(frame, (width, height))
        # Apply effects...
        return result
```

## 📜 License

MIT License - Feel free to use, modify, and distribute!

## 🙏 Credits

Built with:
- [OpenCV](https://opencv.org/) - Video processing
- [Blessed](https://blessed.readthedocs.io/) - Terminal graphics
- [Pygame](https://www.pygame.org/) - Audio playback
- [NumPy](https://numpy.org/) - Array operations

## 🚀 Future Enhancements

- [ ] Support for webcam input
- [ ] Record terminal output to file
- [ ] More render modes (edge detection, color filters, etc.)
- [ ] Playlist support
- [ ] Adjustable playback speed
- [ ] Screenshot capture
- [ ] Custom color palettes
- [ ] Frame interpolation for smoother playback

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Add new render modes
- Improve performance
- Fix bugs
- Enhance documentation

## 📧 Support

If you encounter any issues or have questions, please open an issue on the project repository.

---

**Enjoy your hacker-themed video experience! 🎬💚**
