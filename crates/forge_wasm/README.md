# Forge WASM

WebAssembly bindings for Forge GPU kernels, enabling ML operations in the browser via WebGPU.

## Features

- **Matrix Multiplication**: Batched GPU-accelerated matrix multiplication
- **Activation Functions**: ReLU and GELU implementations
- **Normalization**: RMSNorm layer normalization
- **Performance Metrics**: Built-in benchmarking capabilities
- **WebGPU Detection**: Browser capability checking

## Building

### Prerequisites

1. Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/):
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. Ensure you have Rust installed with the `wasm32-unknown-unknown` target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Build Commands

**Development build** (with debug symbols):
```bash
cd crates/forge_wasm
wasm-pack build --target web --dev
```

**Production build** (optimized for size):
```bash
cd crates/forge_wasm
wasm-pack build --target web --release
```

**Build for Node.js** (if needed):
```bash
wasm-pack build --target nodejs --release
```

**Build for bundlers** (webpack, etc.):
```bash
wasm-pack build --target bundler --release
```

### Output

The build creates a `pkg/` directory containing:
- `forge_wasm.js` - JavaScript bindings
- `forge_wasm_bg.wasm` - WebAssembly binary
- `forge_wasm.d.ts` - TypeScript declarations
- `package.json` - NPM package configuration

## Usage

### In the Browser

```javascript
import init, { ForgeWasm, is_webgpu_supported } from './pkg/forge_wasm.js';

// Check WebGPU support
if (!is_webgpu_supported()) {
    console.error('WebGPU not supported in this browser');
}

// Initialize
await init();
const forge = await ForgeWasm.init();

// Get adapter info
const info = forge.get_adapter_info();
console.log('GPU:', info.name, info.backend);

// Run matrix multiplication
const config = { batch: 1, m: 4, k: 4, n: 4 };
const a = new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
const b = new Float32Array([2, 0, 0, 0, 0, 3, 0, 0, 0, 0, 4, 0, 0, 0, 0, 5]);

const result = await forge.matmul(a, b, config);
console.log('Result:', result);

// Run with metrics
const { result: matmulResult, metrics } = await forge.matmul_with_metrics(a, b, config);
console.log('Duration:', metrics.duration_ms, 'ms');

// Activation functions
const input = new Float32Array([-2, -1, 0, 1, 2]);
const reluResult = await forge.activation(input, 'relu');
const geluResult = await forge.activation(input, 'gelu');

// RMSNorm
const norm_input = new Float32Array([1, 2, 3, 4]);
const weights = new Float32Array([1, 1, 1, 1]);
const normalized = await forge.rmsnorm(norm_input, weights, 1e-5);

// Run benchmarks
const benchmarks = await forge.benchmark();
benchmarks.forEach(b => {
    console.log(`${b.kernel_name}: ${b.duration_ms.toFixed(2)}ms`);
});
```

### In React/TypeScript

```typescript
import init, { ForgeWasm, MatMulConfig } from '@forge/wasm';

const WebGPUComponent = () => {
    const [forge, setForge] = useState<ForgeWasm | null>(null);

    useEffect(() => {
        (async () => {
            await init();
            const instance = await ForgeWasm.init();
            setForge(instance);
        })();
    }, []);

    const runCompute = async () => {
        if (!forge) return;

        const config = new MatMulConfig(1, 8, 8, 8);
        const a = new Float32Array(64).fill(1);
        const b = new Float32Array(64).fill(2);

        const result = await forge.matmul(a, b, config);
        console.log('Computed:', result);
    };

    return (
        <button onClick={runCompute} disabled={!forge}>
            Run GPU Compute
        </button>
    );
};
```

## API Reference

### `ForgeWasm.init(): Promise<ForgeWasm>`
Initialize the Forge WASM module with WebGPU context.

### `forge.get_adapter_info(): object`
Get GPU adapter information (name, vendor, device type, backend).

### `forge.matmul(a: Float32Array, b: Float32Array, config: MatMulConfig): Promise<Float32Array>`
Perform batched matrix multiplication on GPU.

### `forge.matmul_with_metrics(a, b, config): Promise<{ result, metrics }>`
Run matmul with performance timing.

### `forge.activation(input: Float32Array, type: 'relu' | 'gelu'): Promise<Float32Array>`
Apply activation function on GPU.

### `forge.rmsnorm(input: Float32Array, weight: Float32Array, epsilon: number): Promise<Float32Array>`
Apply RMS normalization on GPU.

### `forge.benchmark(): Promise<PerformanceMetrics[]>`
Run comprehensive benchmark of all kernels.

### `is_webgpu_supported(): boolean`
Check if WebGPU is available in the browser.

### `get_webgpu_info(): Promise<object>`
Get detailed WebGPU capability information.

## Browser Support

WebGPU is required and is currently supported in:
- Chrome/Edge 113+ (with flag enabled or native support)
- Firefox Nightly (with flag enabled)
- Safari Technology Preview

Enable WebGPU in browsers:
- **Chrome/Edge**: `chrome://flags/#enable-unsafe-webgpu`
- **Firefox**: `about:config` → `dom.webgpu.enabled`
- **Safari**: Enabled by default in Technology Preview

## Performance

Typical performance on a modern GPU (RTX 3070):
- MatMul (8×8×8): ~2-5ms (including GPU transfer)
- ReLU (1024): ~0.5-1ms
- GELU (1024): ~1-2ms
- RMSNorm (512): ~1-3ms

Note: First run may be slower due to shader compilation and GPU warmup.

## Integration with Forge UI

The Forge web UI automatically detects and uses the WASM module when available:

```bash
# Build WASM module
cd crates/forge_wasm
wasm-pack build --target web --release

# Copy to UI public directory
cp -r pkg ../../ui/app/public/wasm

# The UI will load it automatically
cd ../../ui/app
npm run dev
```

## Troubleshooting

**"WebGPU not supported"**
- Ensure you're using a compatible browser
- Enable WebGPU flags if needed
- Check `navigator.gpu` is available in console

**"Failed to create GPU context"**
- Check if GPU drivers are up to date
- Try disabling hardware acceleration and re-enabling
- Check browser console for detailed error messages

**Large WASM file size**
- Use `--release` build for production
- The WASM is compressed with Brotli during HTTP transfer
- Typical size: ~200-300KB (compressed: ~80-100KB)

**Slow performance**
- First run compiles shaders; subsequent runs are faster
- Ensure GPU acceleration is enabled in browser
- Check if fallback software rendering is being used

## Development

Run tests (requires WebGPU-capable browser):
```bash
wasm-pack test --headless --chrome
```

Build with verbose logging:
```bash
RUST_LOG=debug wasm-pack build --target web --dev
```

## License

MIT OR Apache-2.0
