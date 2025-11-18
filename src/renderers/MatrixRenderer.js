import { BaseRenderer } from './BaseRenderer.js';

export class MatrixRenderer extends BaseRenderer {
  constructor() {
    super('Matrix Rain');
    this.drops = [];
    this.matrixChars = 'ﾊﾐﾋｰｳｼﾅﾓﾆｻﾜﾂｵﾘｱﾎﾃﾏｹﾒｴｶｷﾑﾕﾗｾﾈｽﾀﾇﾍ0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    this.initialized = false;
  }

  initializeDrops(width, height) {
    this.drops = [];
    for (let i = 0; i < width; i++) {
      this.drops.push({
        y: Math.floor(Math.random() * height),
        speed: Math.random() * 0.5 + 0.3,
        chars: []
      });
    }
    this.initialized = true;
  }

  async render(image, termWidth, termHeight) {
    if (!image) return '';

    if (!this.initialized || this.drops.length !== termWidth) {
      this.initializeDrops(termWidth, termHeight);
    }

    // Sample the image
    const sampleWidth = Math.floor(image.bitmap.width / termWidth);
    const sampleHeight = Math.floor(image.bitmap.height / termHeight);

    let output = '';

    for (let y = 0; y < termHeight; y++) {
      for (let x = 0; x < termWidth; x++) {
        const drop = this.drops[x];

        // Calculate if this position should have a character
        const distance = Math.abs(y - drop.y);
        const trailLength = 10 + (this.parameter / 100) * 20; // Trail length based on parameter

        if (distance < trailLength) {
          // Sample color from video at this position
          const imgX = Math.min(x * sampleWidth, image.bitmap.width - 1);
          const imgY = Math.min(y * sampleHeight, image.bitmap.height - 1);
          const color = image.getPixelColor(imgX, imgY);
          const rgba = image.constructor.intToRGBA(color);

          const brightness = this.getBrightness(rgba.r, rgba.g, rgba.b);

          // Fade based on distance from drop head
          const fade = 1 - (distance / trailLength);
          const r = Math.floor(rgba.r * fade);
          const g = Math.floor(rgba.g * fade * (brightness > 0.3 ? 1.5 : 1)); // Emphasize green
          const b = Math.floor(rgba.b * fade);

          // Pick a random character
          const char = this.matrixChars[Math.floor(Math.random() * this.matrixChars.length)];

          if (distance === 0) {
            // Head of the drop - make it bright
            output += this.rgbToAnsi(255, 255, 255) + char + this.resetAnsi();
          } else {
            output += this.rgbToAnsi(r, g, b) + char + this.resetAnsi();
          }
        } else {
          output += ' ';
        }
      }
      output += '\n';
    }

    // Update drops
    for (let drop of this.drops) {
      drop.y += drop.speed;
      if (drop.y > termHeight + 10) {
        drop.y = -10;
        drop.speed = Math.random() * 0.5 + 0.3;
      }
    }

    return output;
  }
}
