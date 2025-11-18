#!/usr/bin/env node

import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { App } from './App.js';
import { existsSync } from 'fs';

const argv = yargs(hideBin(process.argv))
  .usage('Usage: $0 <video_file> [options]')
  .command('$0 <video_file>', 'Play a video in the terminal', (yargs) => {
    yargs.positional('video_file', {
      describe: 'Path to the video file to play',
      type: 'string'
    });
  })
  .option('mode', {
    alias: 'm',
    describe: 'Initial render mode',
    choices: ['ascii', 'pixelate', 'matrix', 'pulse'],
    default: 'ascii'
  })
  .help('h')
  .alias('h', 'help')
  .example('$0 video.mp4', 'Play video.mp4 with default ASCII mode')
  .example('$0 video.mp4 --mode=matrix', 'Play with Matrix rain effect')
  .example('$0 video.mp4 -m pixelate', 'Play with pixelate effect')
  .epilog(`
Controls:
  SPACE    Play/Pause
  Q        Quit
  ← →      Seek backward/forward (5 seconds)
  M        Cycle through render modes
  + -      Increase/decrease mode parameter

Render Modes:
  ascii     ASCII art rendering with colored characters
  pixelate  Retro pixelated blocks
  matrix    Matrix-style falling rain effect
  pulse     Ambient color pulsing based on video colors
`)
  .argv;

async function main() {
  const videoFile = argv.video_file || argv._[0];

  if (!videoFile) {
    console.error('Error: Please provide a video file');
    console.error('Usage: vid-player <video_file> [--mode=<mode>]');
    process.exit(1);
  }

  if (!existsSync(videoFile)) {
    console.error(`Error: Video file not found: ${videoFile}`);
    process.exit(1);
  }

  console.log('Initializing video player...');
  console.log(`Video: ${videoFile}`);
  console.log(`Mode: ${argv.mode}`);
  console.log('');

  const app = new App(videoFile, argv.mode);

  try {
    const metadata = await app.initialize();
    console.log(`Duration: ${Math.floor(metadata.duration)}s`);
    console.log(`Resolution: ${metadata.width}x${metadata.height}`);
    console.log(`Frame rate: ${Math.floor(metadata.frameRate)} FPS`);
    console.log('');
    console.log('Starting playback...');

    // Small delay to let user see the info
    await new Promise(resolve => setTimeout(resolve, 1000));

    await app.start();
  } catch (error) {
    console.error('Error:', error.message);
    await app.cleanup();
    process.exit(1);
  }
}

main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});
