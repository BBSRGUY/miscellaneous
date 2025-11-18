/**
 * Main Piano Application
 * Integrates audio and visual engines with user input
 */

class Piano {
    constructor() {
        this.audioEngine = new AudioEngine();
        this.visualEngine = null;
        this.currentOctave = 4;
        this.activeKeys = new Set(); // Track currently pressed keyboard keys

        // Keyboard to piano key mapping
        this.keyMap = {
            // White keys - lower row (C4-B4)
            'a': 'C',
            's': 'D',
            'd': 'E',
            'f': 'F',
            'g': 'G',
            'h': 'A',
            'j': 'B',
            // White keys - upper row (C5-G5)
            'q': 'C',
            'w': 'D',
            'e': 'E',
            'r': 'F',
            't': 'G',
            'y': 'A',
            'u': 'B',
            'i': 'C',
            'o': 'D',
            'p': 'E',
            // Black keys
            '2': 'C#',
            '3': 'D#',
            '5': 'F#',
            '6': 'G#',
            '7': 'A#',
            '9': 'C#',
            '0': 'D#'
        };

        // Octave offsets for different rows
        this.octaveOffsets = {
            'a': 0, 's': 0, 'd': 0, 'f': 0, 'g': 0, 'h': 0, 'j': 0,
            '2': 0, '3': 0, '5': 0, '6': 0, '7': 0,
            'q': 1, 'w': 1, 'e': 1, 'r': 1, 't': 1, 'y': 1, 'u': 1, 'i': 2, 'o': 2, 'p': 2,
            '9': 1, '0': 1
        };

        this.init();
    }

    /**
     * Initialize the application
     */
    init() {
        // Wait for user interaction to start
        const startScreen = document.getElementById('start-screen');
        const startBtn = document.getElementById('start-btn');

        startBtn.addEventListener('click', () => {
            this.start();
            startScreen.classList.add('hidden');
        });
    }

    /**
     * Start the application
     */
    start() {
        // Initialize audio engine
        this.audioEngine.init();

        // Initialize visual engine
        const container = document.getElementById('canvas-container');
        this.visualEngine = new VisualEngine(container);

        // Setup UI controls
        this.setupControls();

        // Setup keyboard input
        this.setupKeyboardInput();

        console.log('Piano application started!');
    }

    /**
     * Setup UI controls
     */
    setupControls() {
        // Volume control
        const volumeSlider = document.getElementById('volume-slider');
        const volumeValue = document.getElementById('volume-value');

        volumeSlider.addEventListener('input', (e) => {
            const value = e.target.value / 100;
            this.audioEngine.setVolume(value);
            volumeValue.textContent = `${e.target.value}%`;
        });

        // Waveform selection
        const waveformSelect = document.getElementById('waveform-select');
        waveformSelect.addEventListener('change', (e) => {
            this.audioEngine.setWaveform(e.target.value);
        });

        // Visual style buttons
        const styleBtns = document.querySelectorAll('.style-btn');
        styleBtns.forEach(btn => {
            btn.addEventListener('click', (e) => {
                styleBtns.forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                const style = btn.dataset.style;
                this.visualEngine.setStyle(style);
            });
        });

        // Sound engine buttons
        const soundBtns = document.querySelectorAll('.sound-btn');
        soundBtns.forEach(btn => {
            btn.addEventListener('click', (e) => {
                soundBtns.forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                const sound = btn.dataset.sound;
                this.audioEngine.setMode(sound);

                // Show/hide synth controls based on mode
                const synthControls = document.getElementById('synth-controls');
                if (sound === 'synth') {
                    synthControls.style.display = 'flex';
                } else {
                    synthControls.style.display = 'none';
                }
            });
        });
    }

    /**
     * Setup keyboard input handlers
     */
    setupKeyboardInput() {
        document.addEventListener('keydown', (e) => {
            const key = e.key.toLowerCase();

            // Prevent repeat events when key is held down
            if (this.activeKeys.has(key)) return;

            // Handle octave shifting
            if (key === '[') {
                this.changeOctave(-1);
                return;
            }
            if (key === ']') {
                this.changeOctave(1);
                return;
            }

            // Handle sustain pedal
            if (key === ' ') {
                e.preventDefault();
                this.setSustain(true);
                return;
            }

            // Handle piano keys
            if (this.keyMap[key]) {
                e.preventDefault();
                this.activeKeys.add(key);
                this.playNote(key);
            }
        });

        document.addEventListener('keyup', (e) => {
            const key = e.key.toLowerCase();

            // Remove from active keys
            this.activeKeys.delete(key);

            // Handle sustain pedal
            if (key === ' ') {
                e.preventDefault();
                this.setSustain(false);
                return;
            }

            // Handle piano keys
            if (this.keyMap[key]) {
                e.preventDefault();
                this.stopNote(key);
            }
        });

        // Stop all notes when window loses focus
        window.addEventListener('blur', () => {
            this.audioEngine.stopAllNotes();
            this.activeKeys.clear();
            this.setSustain(false);
        });
    }

    /**
     * Play a note based on keyboard key
     */
    playNote(key) {
        const note = this.keyMap[key];
        const octaveOffset = this.octaveOffsets[key] || 0;
        const octave = this.currentOctave + octaveOffset;
        const noteName = `${note}${octave}`;

        // Play audio
        this.audioEngine.playNote(noteName);

        // Animate visual
        this.visualEngine.pressKey(noteName);

        console.log(`Playing: ${noteName}`);
    }

    /**
     * Stop a note based on keyboard key
     */
    stopNote(key) {
        const note = this.keyMap[key];
        const octaveOffset = this.octaveOffsets[key] || 0;
        const octave = this.currentOctave + octaveOffset;
        const noteName = `${note}${octave}`;

        // Stop audio
        this.audioEngine.stopNote(noteName);

        // Animate visual
        this.visualEngine.releaseKey(noteName);
    }

    /**
     * Change the current octave
     */
    changeOctave(delta) {
        const newOctave = this.currentOctave + delta;

        // Limit octave range
        if (newOctave < 0 || newOctave > 7) return;

        this.currentOctave = newOctave;

        // Update display
        const octaveDisplay = document.getElementById('octave-display');
        octaveDisplay.textContent = this.currentOctave;

        console.log(`Octave changed to: ${this.currentOctave}`);
    }

    /**
     * Set sustain pedal state
     */
    setSustain(enabled) {
        this.audioEngine.setSustain(enabled);

        // Update UI
        const sustainIndicator = document.getElementById('sustain-indicator');
        if (enabled) {
            sustainIndicator.classList.remove('off');
            sustainIndicator.classList.add('on');
            sustainIndicator.textContent = 'ON';
        } else {
            sustainIndicator.classList.remove('on');
            sustainIndicator.classList.add('off');
            sustainIndicator.textContent = 'OFF';
        }
    }
}

// Initialize the piano application when the page loads
document.addEventListener('DOMContentLoaded', () => {
    const piano = new Piano();
});
