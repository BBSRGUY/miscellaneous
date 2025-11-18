# Quick Start Guide

Get up and running with Hacker Terminal Video Player in 3 steps!

## Step 1: Install Dependencies

```bash
cd hacker-terminal-player
pip install -r requirements.txt
```

## Step 2: Create a Demo Video

```bash
python create_demo_video.py
```

This will generate a test video at `examples/demo.mp4`.

## Step 3: Play the Video!

```bash
./vid-player examples/demo.mp4
```

## Controls Cheat Sheet

Once playing:
- **SPACE** - Play/Pause
- **M** - Switch visual modes
- **+/-** - Adjust effect intensity
- **←/→** - Seek backward/forward
- **Q** - Quit

## Try Different Modes

```bash
# Pixelated retro style
./vid-player examples/demo.mp4 --mode=pixelate

# ASCII art style
./vid-player examples/demo.mp4 --mode=ascii

# Glitch effects
./vid-player examples/demo.mp4 --mode=glitch

# Matrix-style display
./vid-player examples/demo.mp4 --mode=matrix
```

## Tips

- **Maximize your terminal** for the best experience
- **Use a dark terminal theme** for the authentic hacker look
- **Press M during playback** to cycle through all modes
- **Adjust parameters with +/-** to fine-tune each effect

## Troubleshooting

**"Video file not found"**
- Make sure you ran `python create_demo_video.py` first
- Or use your own video: `./vid-player path/to/your/video.mp4`

**"Module not found" errors**
- Run `pip install -r requirements.txt` again
- Make sure you're using Python 3.8+

**Colors look weird**
- Your terminal might not support 256 colors
- Try using a modern terminal like iTerm2, Windows Terminal, or GNOME Terminal

**Player is slow/laggy**
- Make your terminal window smaller
- Try ASCII mode (usually fastest)
- Lower the effect parameter with the `-` key

---

Enjoy your hacker video experience! 💚
