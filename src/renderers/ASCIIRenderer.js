import { BaseRenderer } from './BaseRenderer.js';

export class ASCIIRenderer extends BaseRenderer {
  constructor() {
    super('ASCII Art');
    // ASCII characters from darkest to lightest
    this.charSets = {
      simple: ' .:-=+*#%@',
      detailed: ' .\':,;!~_-+=?/<>|\\()[]{}I1ltfjrxnuvczXYUJCLQ0OZmwqpdbkhaoS#@',
      blocks: ' ░▒▓█'
    };
    this.currentCharSet = 'detailed';
  }

  async render(image, termWidth, termHeight) {
    if (!image) return '';

    const chars = this.charSets[this.currentCharSet];

    // Adjust character density based on parameter (0-100)
    const density = this.parameter / 100;
    const effectiveChars = chars.slice(0, Math.max(2, Math.floor(chars.length * density)));

    // Calculate sampling rate to fit terminal
    const aspectRatio = 0.5; // Characters are taller than wide
    const sampleWidth = Math.floor(image.bitmap.width / termWidth);
    const sampleHeight = Math.floor(image.bitmap.height / (termHeight * aspectRatio));

    let output = '';

    for (let y = 0; y < image.bitmap.height; y += sampleHeight) {
      for (let x = 0; x < image.bitmap.width; x += sampleWidth) {
        if (x >= image.bitmap.width || y >= image.bitmap.height) continue;

        const color = image.getPixelColor(x, y);
        const rgba = image.constructor.intToRGBA(color);

        const brightness = this.getBrightness(rgba.r, rgba.g, rgba.b);
        const charIndex = Math.floor(brightness * (effectiveChars.length - 1));
        const char = effectiveChars[charIndex];

        // Add color
        output += this.rgbToAnsi(rgba.r, rgba.g, rgba.b) + char + this.resetAnsi();
      }
      output += '\n';
    }

    return output;
  }

  // Cycle through character sets
  cycleCharSet() {
    const sets = Object.keys(this.charSets);
    const currentIndex = sets.indexOf(this.currentCharSet);
    this.currentCharSet = sets[(currentIndex + 1) % sets.length];
  }
}
