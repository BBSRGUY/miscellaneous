import { BaseRenderer } from './BaseRenderer.js';

export class PixelateRenderer extends BaseRenderer {
  constructor() {
    super('Pixelate');
    this.blockChars = '█';
  }

  async render(image, termWidth, termHeight) {
    if (!image) return '';

    // Parameter controls pixel size (0-100 -> 1-20 blocks)
    const minBlockSize = 1;
    const maxBlockSize = 20;
    const blockSize = Math.max(minBlockSize, Math.floor((1 - this.parameter / 100) * maxBlockSize + minBlockSize));

    // Calculate how many blocks we can fit
    const blocksWide = Math.floor(termWidth / blockSize);
    const blocksHigh = Math.floor(termHeight / blockSize);

    const sampleWidth = Math.floor(image.bitmap.width / blocksWide);
    const sampleHeight = Math.floor(image.bitmap.height / blocksHigh);

    let output = '';

    for (let by = 0; by < blocksHigh; by++) {
      for (let row = 0; row < blockSize; row++) {
        for (let bx = 0; bx < blocksWide; bx++) {
          // Sample the image at this block position
          const imgX = Math.min(bx * sampleWidth, image.bitmap.width - 1);
          const imgY = Math.min(by * sampleHeight, image.bitmap.height - 1);

          // Get average color of the block area
          let r = 0, g = 0, b = 0, count = 0;
          const sampleSize = Math.max(1, Math.floor(sampleWidth / 4));

          for (let sy = 0; sy < sampleHeight; sy += sampleSize) {
            for (let sx = 0; sx < sampleWidth; sx += sampleSize) {
              const px = Math.min(imgX + sx, image.bitmap.width - 1);
              const py = Math.min(imgY + sy, image.bitmap.height - 1);
              const color = image.getPixelColor(px, py);
              const rgba = image.constructor.intToRGBA(color);
              r += rgba.r;
              g += rgba.g;
              b += rgba.b;
              count++;
            }
          }

          r = Math.floor(r / count);
          g = Math.floor(g / count);
          b = Math.floor(b / count);

          // Draw the block
          for (let col = 0; col < blockSize; col++) {
            output += this.rgbToBgAnsi(r, g, b) + ' ' + this.resetAnsi();
          }
        }
        output += '\n';
      }
    }

    return output;
  }
}
