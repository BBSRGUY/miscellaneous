export class BaseRenderer {
  constructor(name) {
    this.name = name;
    this.parameter = 50; // Default parameter value (0-100)
  }

  getName() {
    return this.name;
  }

  increaseParameter() {
    this.parameter = Math.min(100, this.parameter + 10);
  }

  decreaseParameter() {
    this.parameter = Math.max(0, this.parameter - 10);
  }

  getParameter() {
    return this.parameter;
  }

  // Override this method in subclasses
  async render(image, width, height) {
    throw new Error('render() must be implemented by subclass');
  }

  // Utility: Get average color of image
  getAverageColor(image) {
    let r = 0, g = 0, b = 0;
    let count = 0;

    const step = Math.max(1, Math.floor(image.bitmap.width / 20));

    for (let y = 0; y < image.bitmap.height; y += step) {
      for (let x = 0; x < image.bitmap.width; x += step) {
        const color = image.getPixelColor(x, y);
        const rgba = image.constructor.intToRGBA(color);
        r += rgba.r;
        g += rgba.g;
        b += rgba.b;
        count++;
      }
    }

    return {
      r: Math.floor(r / count),
      g: Math.floor(g / count),
      b: Math.floor(b / count)
    };
  }

  // Utility: Convert RGB to ANSI color code
  rgbToAnsi(r, g, b) {
    return `\x1b[38;2;${r};${g};${b}m`;
  }

  // Utility: Convert RGB to ANSI background color
  rgbToBgAnsi(r, g, b) {
    return `\x1b[48;2;${r};${g};${b}m`;
  }

  // Utility: Reset ANSI colors
  resetAnsi() {
    return '\x1b[0m';
  }

  // Utility: Get brightness of a color
  getBrightness(r, g, b) {
    return (r * 0.299 + g * 0.587 + b * 0.114) / 255;
  }
}
