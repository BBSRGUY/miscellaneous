import ffmpeg from 'fluent-ffmpeg';
import Jimp from 'jimp';
import { promises as fs } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export class VideoPlayer {
  constructor(videoPath) {
    this.videoPath = videoPath;
    this.frameRate = 30;
    this.duration = 0;
    this.currentFrame = 0;
    this.totalFrames = 0;
    this.isPlaying = false;
    this.frameBuffer = [];
    this.bufferSize = 60; // Buffer 60 frames ahead
    this.tempDir = path.join(__dirname, '../.tmp');
  }

  async initialize() {
    // Create temp directory for frame extraction
    await fs.mkdir(this.tempDir, { recursive: true });

    // Get video metadata
    return new Promise((resolve, reject) => {
      ffmpeg.ffprobe(this.videoPath, (err, metadata) => {
        if (err) {
          reject(err);
          return;
        }

        const videoStream = metadata.streams.find(s => s.codec_type === 'video');
        if (!videoStream) {
          reject(new Error('No video stream found'));
          return;
        }

        this.frameRate = eval(videoStream.r_frame_rate) || 30;
        this.duration = parseFloat(metadata.format.duration);
        this.totalFrames = Math.floor(this.duration * this.frameRate);
        this.width = videoStream.width;
        this.height = videoStream.height;

        resolve({
          duration: this.duration,
          frameRate: this.frameRate,
          totalFrames: this.totalFrames,
          width: this.width,
          height: this.height
        });
      });
    });
  }

  async extractFrame(frameNumber) {
    const timestamp = frameNumber / this.frameRate;
    const outputPath = path.join(this.tempDir, `frame_${frameNumber}.jpg`);

    // Check if frame already exists
    try {
      await fs.access(outputPath);
      return outputPath;
    } catch {
      // Frame doesn't exist, extract it
    }

    return new Promise((resolve, reject) => {
      ffmpeg(this.videoPath)
        .seekInput(timestamp)
        .frames(1)
        .output(outputPath)
        .on('end', () => resolve(outputPath))
        .on('error', reject)
        .run();
    });
  }

  async getFrameData(frameNumber) {
    try {
      const framePath = await this.extractFrame(frameNumber);
      const image = await Jimp.read(framePath);
      return image;
    } catch (error) {
      console.error(`Error getting frame ${frameNumber}:`, error);
      return null;
    }
  }

  async seek(frameNumber) {
    this.currentFrame = Math.max(0, Math.min(frameNumber, this.totalFrames - 1));
    this.frameBuffer = [];
  }

  async seekForward(seconds = 5) {
    const frames = Math.floor(seconds * this.frameRate);
    await this.seek(this.currentFrame + frames);
  }

  async seekBackward(seconds = 5) {
    const frames = Math.floor(seconds * this.frameRate);
    await this.seek(this.currentFrame - frames);
  }

  async nextFrame() {
    if (this.currentFrame < this.totalFrames - 1) {
      this.currentFrame++;
      return await this.getFrameData(this.currentFrame);
    }
    return null;
  }

  play() {
    this.isPlaying = true;
  }

  pause() {
    this.isPlaying = false;
  }

  toggle() {
    this.isPlaying = !this.isPlaying;
  }

  getProgress() {
    return {
      current: this.currentFrame,
      total: this.totalFrames,
      percentage: (this.currentFrame / this.totalFrames) * 100,
      time: this.currentFrame / this.frameRate,
      duration: this.duration
    };
  }

  async cleanup() {
    try {
      // Clean up temp directory
      const files = await fs.readdir(this.tempDir);
      for (const file of files) {
        await fs.unlink(path.join(this.tempDir, file));
      }
      await fs.rmdir(this.tempDir);
    } catch (error) {
      // Ignore cleanup errors
    }
  }
}
