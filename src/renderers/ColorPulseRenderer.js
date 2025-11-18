import { BaseRenderer } from './BaseRenderer.js';

export class ColorPulseRenderer extends BaseRenderer {
  constructor() {
    super('Color Pulse');
    this.previousColor = { r: 0, g: 0, b: 0 };
    this.currentColor = { r: 0, g: 0, b: 0 };
    this.pulsePhase = 0;
    this.smoothing = 0.3; // Smoothing factor for color transitions
  }

  async render(image, termWidth, termHeight) {
    if (!image) return '';

    // Get average color of the frame
    const avgColor = this.getAverageColor(image);

    // Smooth transition from previous to current color
    this.currentColor.r = Math.floor(this.currentColor.r * this.smoothing + avgColor.r * (1 - this.smoothing));
    this.currentColor.g = Math.floor(this.currentColor.g * this.smoothing + avgColor.g * (1 - this.smoothing));
    this.currentColor.b = Math.floor(this.currentColor.b * this.smoothing + avgColor.b * (1 - this.smoothing));

    // Update pulse phase
    const pulseSpeed = 0.1 + (this.parameter / 100) * 0.3; // Pulse speed based on parameter
    this.pulsePhase += pulseSpeed;
    const pulseFactor = (Math.sin(this.pulsePhase) + 1) / 2; // 0 to 1

    // Apply pulse to brightness
    const pulseR = Math.floor(this.currentColor.r * (0.5 + pulseFactor * 0.5));
    const pulseG = Math.floor(this.currentColor.g * (0.5 + pulseFactor * 0.5));
    const pulseB = Math.floor(this.currentColor.b * (0.5 + pulseFactor * 0.5));

    // Create a pattern based on the pulse
    let output = '';
    const centerX = Math.floor(termWidth / 2);
    const centerY = Math.floor(termHeight / 2);

    for (let y = 0; y < termHeight; y++) {
      for (let x = 0; x < termWidth; x++) {
        // Calculate distance from center
        const dx = x - centerX;
        const dy = (y - centerY) * 2; // Account for character aspect ratio
        const distance = Math.sqrt(dx * dx + dy * dy);
        const maxDistance = Math.sqrt(centerX * centerX + (centerY * 2) * (centerY * 2));
        const normalizedDistance = distance / maxDistance;

        // Create ripple effect
        const ripple = Math.sin(normalizedDistance * 10 - this.pulsePhase * 2);
        const intensity = (ripple + 1) / 2;

        const r = Math.floor(pulseR * intensity);
        const g = Math.floor(pulseG * intensity);
        const b = Math.floor(pulseB * intensity);

        // Choose character based on intensity
        const chars = ' .·:•○●◉';
        const charIndex = Math.floor(intensity * (chars.length - 1));
        const char = chars[charIndex];

        output += this.rgbToAnsi(r, g, b) + char + this.resetAnsi();
      }
      output += '\n';
    }

    return output;
  }
}
