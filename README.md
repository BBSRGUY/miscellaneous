# 🎹 3D Web Piano

An immersive 3D piano experience powered by Three.js and the Web Audio API. Play beautiful music with your keyboard in a stunning visual environment.

## Features

### 🎵 Sound Engine
- **Synthesizer Mode**: Real-time sound synthesis with multiple waveforms (sine, square, sawtooth, triangle)
- **Piano Mode**: Simulated piano timbre using harmonic synthesis
- **Web Audio API**: High-quality, low-latency audio playback
- **Volume Control**: Adjustable master volume
- **Sustain Pedal**: Hold notes with the spacebar

### 🌟 Visual Styles
- **Neon Grid**: Synthwave-inspired aesthetic with glowing keys and grid
- **Classic**: Traditional piano appearance
- **Particles**: Particle burst effects for each note played

### 🎹 Piano Controls
- **Two Full Octaves**: Play C4-E6 with your keyboard
- **Octave Shifting**: Use `[` and `]` keys to shift octaves up and down
- **Sustain Pedal**: Press `Space` to enable sustain mode
- **Real-time Key Animation**: Visual feedback for every key press

### ⌨️ Keyboard Layout

**White Keys (Lower Octave):**
- `A S D F G H J` = C4 D4 E4 F4 G4 A4 B4

**White Keys (Upper Octave):**
- `Q W E R T Y U I O P` = C5-E6

**Black Keys:**
- `2 3 5 6 7` = C#4 D#4 F#4 G#4 A#4
- `9 0` = C#5 D#5

**Special Keys:**
- `[` = Octave Down
- `]` = Octave Up
- `Space` = Sustain Pedal

## Installation & Setup

### Prerequisites
- Node.js (v14 or higher)
- npm

### Quick Start

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Start the server:**
   ```bash
   npm start
   ```

3. **Open your browser:**
   Navigate to `http://localhost:3000`

4. **Click "Start" to begin playing!**

## Architecture

### Backend (Node.js)
- **Express.js**: Lightweight web server serving static files
- Serves HTML, CSS, JavaScript, and assets

### Frontend (Browser)
- **Three.js**: 3D rendering engine for visual piano
- **Web Audio API**: Real-time audio synthesis and playback
- **Vanilla JavaScript**: Application logic and input handling

## Project Structure

```
web-3d-piano/
├── server.js                 # Express server
├── package.json             # Node.js dependencies
├── public/                  # Static files
│   ├── index.html          # Main HTML page
│   ├── css/
│   │   └── style.css       # Styling and UI
│   └── js/
│       ├── piano.js        # Main application logic
│       ├── audio-engine.js # Web Audio API implementation
│       └── visual-engine.js # Three.js 3D rendering
└── README.md
```

## Technical Details

### Audio Engine
- **Frequency Calculation**: Equal temperament tuning (A4 = 440 Hz)
- **ADSR Envelope**: Attack, Decay, Sustain, Release for natural sound
- **Harmonic Synthesis**: Multiple oscillators for rich piano timbre
- **Filter Processing**: Lowpass filters for tone shaping

### Visual Engine
- **3D Piano Model**: Procedurally generated with correct proportions
- **Dynamic Lighting**: Point lights and directional lights for atmosphere
- **Particle System**: Real-time particle effects for visual feedback
- **Smooth Animations**: Keypress animations with interpolation

## Browser Compatibility

Works best in modern browsers with WebGL and Web Audio API support:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Development

### Customization
You can customize the piano experience by modifying:

- **Audio Engine** (`audio-engine.js`): Adjust waveforms, envelopes, and effects
- **Visual Engine** (`visual-engine.js`): Change 3D models, lighting, and animations
- **Styles** (`style.css`): Modify UI appearance and colors

### Adding New Visual Styles
1. Add a new style button in `index.html`
2. Create a new style method in `visual-engine.js`
3. Update the style switcher in `piano.js`

### Adding New Sound Modes
1. Create a new playback method in `audio-engine.js`
2. Add UI controls in `index.html`
3. Wire up the controls in `piano.js`

## License

MIT License - Feel free to use and modify for your projects!

## Credits

Built with:
- [Three.js](https://threejs.org/) - 3D graphics library
- [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API) - Audio synthesis
- [Express.js](https://expressjs.com/) - Web server

---

Enjoy making music! 🎶
