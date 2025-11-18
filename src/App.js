import { VideoPlayer } from './VideoPlayer.js';
import { TerminalUI } from './TerminalUI.js';
import { ASCIIRenderer } from './renderers/ASCIIRenderer.js';
import { MatrixRenderer } from './renderers/MatrixRenderer.js';
import { ColorPulseRenderer } from './renderers/ColorPulseRenderer.js';
import { PixelateRenderer } from './renderers/PixelateRenderer.js';

export class App {
  constructor(videoPath, initialMode = 'ascii') {
    this.videoPath = videoPath;
    this.player = new VideoPlayer(videoPath);
    this.ui = new TerminalUI();

    // Initialize renderers
    this.renderers = [
      new ASCIIRenderer(),
      new PixelateRenderer(),
      new MatrixRenderer(),
      new ColorPulseRenderer()
    ];

    this.currentRendererIndex = 0;
    this.setRendererByName(initialMode);

    this.isRunning = false;
    this.frameInterval = null;
    this.targetFPS = 30;
  }

  setRendererByName(name) {
    const lowerName = name.toLowerCase();
    const index = this.renderers.findIndex(r =>
      r.getName().toLowerCase().includes(lowerName)
    );
    if (index !== -1) {
      this.currentRendererIndex = index;
    }
  }

  getCurrentRenderer() {
    return this.renderers[this.currentRendererIndex];
  }

  cycleRenderer() {
    this.currentRendererIndex = (this.currentRendererIndex + 1) % this.renderers.length;
    this.updateStatus();
  }

  async initialize() {
    try {
      const metadata = await this.player.initialize();
      this.targetFPS = Math.min(metadata.frameRate, 30); // Cap at 30 FPS for terminal
      this.setupKeyBindings();
      this.updateStatus();
      return metadata;
    } catch (error) {
      throw new Error(`Failed to initialize video: ${error.message}`);
    }
  }

  setupKeyBindings() {
    // Quit
    this.ui.onKey(['q', 'Q', 'C-c'], () => {
      this.cleanup();
      process.exit(0);
    });

    // Play/Pause
    this.ui.onKey(['space'], () => {
      this.player.toggle();
      this.updateStatus();
    });

    // Seek forward
    this.ui.onKey(['right'], async () => {
      await this.player.seekForward(5);
      this.updateStatus();
    });

    // Seek backward
    this.ui.onKey(['left'], async () => {
      await this.player.seekBackward(5);
      this.updateStatus();
    });

    // Cycle render mode
    this.ui.onKey(['m', 'M'], () => {
      this.cycleRenderer();
    });

    // Increase parameter
    this.ui.onKey(['+', '='], () => {
      this.getCurrentRenderer().increaseParameter();
      this.updateStatus();
    });

    // Decrease parameter
    this.ui.onKey(['-', '_'], () => {
      this.getCurrentRenderer().decreaseParameter();
      this.updateStatus();
    });
  }

  updateStatus() {
    const progress = this.player.getProgress();
    const renderer = this.getCurrentRenderer();
    const status = `${this.player.isPlaying ? '▶' : '⏸'} ` +
      `${this.formatTime(progress.time)} / ${this.formatTime(progress.duration)} ` +
      `[${Math.floor(progress.percentage)}%] ` +
      `Mode: ${renderer.getName()} ` +
      `Param: ${renderer.getParameter()}`;
    this.ui.updateStatus(status);
  }

  formatTime(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  async renderFrame() {
    if (!this.player.isPlaying) {
      return;
    }

    const frame = await this.player.nextFrame();
    if (!frame) {
      this.player.pause();
      this.updateStatus();
      return;
    }

    const { width, height } = this.ui.getCharDimensions();
    const renderer = this.getCurrentRenderer();

    try {
      const output = await renderer.render(frame, width, height);
      this.ui.setContent(output);
      this.updateStatus();
      this.ui.render();
    } catch (error) {
      console.error('Render error:', error);
    }
  }

  async start() {
    this.isRunning = true;
    this.player.play();

    // Initial render
    await this.renderFrame();

    // Start render loop
    const frameTime = 1000 / this.targetFPS;
    this.frameInterval = setInterval(() => {
      if (this.isRunning) {
        this.renderFrame();
      }
    }, frameTime);
  }

  async cleanup() {
    this.isRunning = false;
    if (this.frameInterval) {
      clearInterval(this.frameInterval);
    }
    await this.player.cleanup();
    this.ui.destroy();
  }
}
