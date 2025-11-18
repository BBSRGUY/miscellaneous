/**
 * Audio Engine using Web Audio API
 * Supports both synthesizer and piano sample modes
 */

class AudioEngine {
    constructor() {
        this.audioContext = null;
        this.masterGain = null;
        this.activeNotes = new Map(); // Track active oscillators
        this.sustainedNotes = new Set(); // Track sustained notes
        this.sustain = false;
        this.volume = 0.5;
        this.waveform = 'sawtooth';
        this.mode = 'synth'; // 'synth' or 'piano'

        // Note frequency mapping (A4 = 440 Hz)
        this.noteFrequencies = this.generateNoteFrequencies();
    }

    /**
     * Initialize the audio context (must be called after user interaction)
     */
    init() {
        if (this.audioContext) return;

        this.audioContext = new (window.AudioContext || window.webkitAudioContext)();

        // Master gain node for volume control
        this.masterGain = this.audioContext.createGain();
        this.masterGain.gain.value = this.volume;
        this.masterGain.connect(this.audioContext.destination);

        console.log('Audio Engine initialized');
    }

    /**
     * Generate frequencies for all notes
     * Uses equal temperament tuning: f = 440 * 2^((n-49)/12)
     * where n is the number of half-steps from A4
     */
    generateNoteFrequencies() {
        const A4 = 440;
        const frequencies = {};

        // Generate for octaves 0-8
        const notes = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

        for (let octave = 0; octave <= 8; octave++) {
            for (let i = 0; i < notes.length; i++) {
                const note = notes[i];
                const noteName = `${note}${octave}`;

                // Calculate half-steps from A4 (A4 is note 49 in MIDI)
                const midiNote = (octave + 1) * 12 + i;
                const halfStepsFromA4 = midiNote - 69; // A4 is MIDI note 69

                // Calculate frequency
                const frequency = A4 * Math.pow(2, halfStepsFromA4 / 12);
                frequencies[noteName] = frequency;
            }
        }

        return frequencies;
    }

    /**
     * Play a note
     * @param {string} noteName - Note name (e.g., 'C4', 'A#5')
     */
    playNote(noteName) {
        if (!this.audioContext) return;

        // Don't play if note is already playing
        if (this.activeNotes.has(noteName)) return;

        const frequency = this.noteFrequencies[noteName];
        if (!frequency) {
            console.warn(`Note ${noteName} not found`);
            return;
        }

        if (this.mode === 'synth') {
            this.playSynthNote(noteName, frequency);
        } else {
            this.playPianoNote(noteName, frequency);
        }
    }

    /**
     * Play a synthesized note
     */
    playSynthNote(noteName, frequency) {
        const currentTime = this.audioContext.currentTime;

        // Create oscillator
        const oscillator = this.audioContext.createOscillator();
        oscillator.type = this.waveform;
        oscillator.frequency.setValueAtTime(frequency, currentTime);

        // Create envelope (ADSR)
        const noteGain = this.audioContext.createGain();
        noteGain.gain.setValueAtTime(0, currentTime);

        // Attack (20ms)
        noteGain.gain.linearRampToValueAtTime(0.3, currentTime + 0.02);

        // Decay to sustain level (100ms)
        noteGain.gain.linearRampToValueAtTime(0.2, currentTime + 0.12);

        // Create a filter for tone shaping
        const filter = this.audioContext.createBiquadFilter();
        filter.type = 'lowpass';
        filter.frequency.setValueAtTime(2000, currentTime);
        filter.Q.setValueAtTime(1, currentTime);

        // Connect the audio graph
        oscillator.connect(filter);
        filter.connect(noteGain);
        noteGain.connect(this.masterGain);

        // Start the oscillator
        oscillator.start(currentTime);

        // Store the oscillator and gain node
        this.activeNotes.set(noteName, {
            oscillator,
            gainNode: noteGain,
            filter
        });
    }

    /**
     * Play a piano note using synthesis (simulated piano timbre)
     */
    playPianoNote(noteName, frequency) {
        const currentTime = this.audioContext.currentTime;

        // Create multiple oscillators for richer timbre
        const fundamentalOsc = this.audioContext.createOscillator();
        fundamentalOsc.type = 'triangle';
        fundamentalOsc.frequency.setValueAtTime(frequency, currentTime);

        const harmonicOsc = this.audioContext.createOscillator();
        harmonicOsc.type = 'sine';
        harmonicOsc.frequency.setValueAtTime(frequency * 2, currentTime);

        const harmonicOsc2 = this.audioContext.createOscillator();
        harmonicOsc2.type = 'sine';
        harmonicOsc2.frequency.setValueAtTime(frequency * 3, currentTime);

        // Create gains for each oscillator
        const fundamentalGain = this.audioContext.createGain();
        fundamentalGain.gain.setValueAtTime(0.5, currentTime);

        const harmonicGain = this.audioContext.createGain();
        harmonicGain.gain.setValueAtTime(0.2, currentTime);

        const harmonicGain2 = this.audioContext.createGain();
        harmonicGain2.gain.setValueAtTime(0.1, currentTime);

        // Master envelope for the note
        const noteGain = this.audioContext.createGain();
        noteGain.gain.setValueAtTime(0, currentTime);
        noteGain.gain.linearRampToValueAtTime(0.4, currentTime + 0.01); // Fast attack
        noteGain.gain.exponentialRampToValueAtTime(0.3, currentTime + 0.1); // Decay

        // Connect oscillators to their gains
        fundamentalOsc.connect(fundamentalGain);
        harmonicOsc.connect(harmonicGain);
        harmonicOsc2.connect(harmonicGain2);

        // Connect gains to note envelope
        fundamentalGain.connect(noteGain);
        harmonicGain.connect(noteGain);
        harmonicGain2.connect(noteGain);

        // Add filter for warmth
        const filter = this.audioContext.createBiquadFilter();
        filter.type = 'lowpass';
        filter.frequency.setValueAtTime(3000, currentTime);
        filter.Q.setValueAtTime(0.5, currentTime);

        noteGain.connect(filter);
        filter.connect(this.masterGain);

        // Start oscillators
        fundamentalOsc.start(currentTime);
        harmonicOsc.start(currentTime);
        harmonicOsc2.start(currentTime);

        // Store all oscillators and nodes
        this.activeNotes.set(noteName, {
            oscillators: [fundamentalOsc, harmonicOsc, harmonicOsc2],
            gainNode: noteGain,
            filter
        });
    }

    /**
     * Stop a note
     * @param {string} noteName - Note name to stop
     */
    stopNote(noteName) {
        if (!this.audioContext) return;

        // If sustain is on, don't stop the note, just mark it as sustained
        if (this.sustain) {
            this.sustainedNotes.add(noteName);
            return;
        }

        this.releaseNote(noteName);
    }

    /**
     * Actually release a note (with envelope)
     */
    releaseNote(noteName) {
        const noteData = this.activeNotes.get(noteName);
        if (!noteData) return;

        const currentTime = this.audioContext.currentTime;
        const releaseTime = 0.1; // 100ms release

        // Fade out
        noteData.gainNode.gain.cancelScheduledValues(currentTime);
        noteData.gainNode.gain.setValueAtTime(noteData.gainNode.gain.value, currentTime);
        noteData.gainNode.gain.linearRampToValueAtTime(0, currentTime + releaseTime);

        // Stop and clean up after release
        setTimeout(() => {
            if (noteData.oscillator) {
                noteData.oscillator.stop();
            }
            if (noteData.oscillators) {
                noteData.oscillators.forEach(osc => osc.stop());
            }
            this.activeNotes.delete(noteName);
            this.sustainedNotes.delete(noteName);
        }, releaseTime * 1000);
    }

    /**
     * Set sustain pedal state
     */
    setSustain(enabled) {
        this.sustain = enabled;

        // If turning off sustain, release all sustained notes
        if (!enabled) {
            this.sustainedNotes.forEach(noteName => {
                this.releaseNote(noteName);
            });
            this.sustainedNotes.clear();
        }
    }

    /**
     * Set master volume
     */
    setVolume(value) {
        this.volume = value;
        if (this.masterGain) {
            this.masterGain.gain.setValueAtTime(value, this.audioContext.currentTime);
        }
    }

    /**
     * Set waveform type for synthesizer
     */
    setWaveform(waveform) {
        this.waveform = waveform;
    }

    /**
     * Set audio mode (synth or piano)
     */
    setMode(mode) {
        this.mode = mode;
    }

    /**
     * Stop all notes
     */
    stopAllNotes() {
        this.activeNotes.forEach((_, noteName) => {
            this.releaseNote(noteName);
        });
        this.activeNotes.clear();
        this.sustainedNotes.clear();
    }
}
