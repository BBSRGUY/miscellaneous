import blessed from 'blessed';

export class TerminalUI {
  constructor() {
    this.screen = blessed.screen({
      smartCSR: true,
      fullUnicode: true,
      dockBorders: true,
      title: 'Video Player'
    });

    // Main display box
    this.displayBox = blessed.box({
      top: 0,
      left: 0,
      width: '100%',
      height: '100%-3',
      content: '',
      tags: true,
      style: {
        fg: 'white',
        bg: 'black'
      }
    });

    // Status bar
    this.statusBar = blessed.box({
      bottom: 2,
      left: 0,
      width: '100%',
      height: 1,
      content: '',
      tags: true,
      style: {
        fg: 'white',
        bg: 'blue'
      }
    });

    // Controls help
    this.helpBar = blessed.box({
      bottom: 0,
      left: 0,
      width: '100%',
      height: 2,
      content: ' [SPACE] Play/Pause  [Q] Quit  [←/→] Seek  [M] Mode  [+/-] Adjust',
      tags: true,
      style: {
        fg: 'yellow',
        bg: 'black'
      }
    });

    this.screen.append(this.displayBox);
    this.screen.append(this.statusBar);
    this.screen.append(this.helpBar);
  }

  getDisplayDimensions() {
    return {
      width: this.displayBox.width,
      height: this.displayBox.height
    };
  }

  setContent(content) {
    this.displayBox.setContent(content);
  }

  updateStatus(text) {
    this.statusBar.setContent(` ${text}`);
  }

  render() {
    this.screen.render();
  }

  onKey(keys, callback) {
    this.screen.key(keys, callback);
  }

  destroy() {
    this.screen.destroy();
  }

  clear() {
    this.displayBox.setContent('');
  }

  // Get actual pixel dimensions
  getCharDimensions() {
    const width = this.screen.width;
    const height = this.screen.height - 3; // Account for status and help bars
    return { width, height };
  }
}
