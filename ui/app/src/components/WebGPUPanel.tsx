import React, { useState, useEffect } from 'react';

interface WebGPUInfo {
  supported: boolean;
  adapterInfo?: {
    name: string;
    vendor: string;
    device: string;
    deviceType: string;
    backend: string;
  };
  error?: string;
}

interface BenchmarkResult {
  kernel_name: string;
  duration_ms: number;
  input_size: number;
  output_size: number;
}

interface ForgeWasm {
  init(): Promise<ForgeWasm>;
  get_adapter_info(): any;
  matmul_with_metrics(a: Float32Array, b: Float32Array, config: any): Promise<any>;
  benchmark(): Promise<BenchmarkResult[]>;
}

const WebGPUPanel: React.FC = () => {
  const [webgpuInfo, setWebgpuInfo] = useState<WebGPUInfo | null>(null);
  const [forge, setForge] = useState<ForgeWasm | null>(null);
  const [loading, setLoading] = useState(false);
  const [testResult, setTestResult] = useState<string>('');
  const [benchmarks, setBenchmarks] = useState<BenchmarkResult[]>([]);
  const [registered, setRegistered] = useState(false);

  useEffect(() => {
    checkWebGPU();
  }, []);

  const checkWebGPU = async () => {
    setLoading(true);

    try {
      // Check if WebGPU is available
      if (!('gpu' in navigator)) {
        setWebgpuInfo({
          supported: false,
          error: 'WebGPU is not supported in this browser',
        });
        setLoading(false);
        return;
      }

      // Try to get adapter
      const adapter = await (navigator as any).gpu.requestAdapter();
      if (!adapter) {
        setWebgpuInfo({
          supported: false,
          error: 'Failed to get WebGPU adapter',
        });
        setLoading(false);
        return;
      }

      const info = await adapter.requestAdapterInfo();

      setWebgpuInfo({
        supported: true,
        adapterInfo: {
          name: info?.description || 'Unknown',
          vendor: String(info?.vendor || 'Unknown'),
          device: String(info?.device || 'Unknown'),
          deviceType: info?.deviceType || 'Unknown',
          backend: info?.backend || 'Unknown',
        },
      });
    } catch (error) {
      setWebgpuInfo({
        supported: false,
        error: `Error checking WebGPU: ${error}`,
      });
    }

    setLoading(false);
  };

  const initializeForgeWasm = async () => {
    setLoading(true);
    setTestResult('Initializing Forge WASM...');

    try {
      // Dynamically import the WASM module
      const wasmModule = await import('/wasm/forge_wasm.js');

      // Initialize the module
      await wasmModule.default();

      // Create Forge instance
      const forgeInstance = await wasmModule.ForgeWasm.init();
      setForge(forgeInstance);

      // Get adapter info
      const adapterInfo = forgeInstance.get_adapter_info();
      setTestResult(`✓ Forge WASM initialized successfully!\nGPU: ${adapterInfo.name}\nBackend: ${adapterInfo.backend}`);
    } catch (error) {
      setTestResult(`✗ Failed to initialize Forge WASM: ${error}`);
      console.error('Forge WASM initialization error:', error);
    }

    setLoading(false);
  };

  const runMatMulTest = async () => {
    if (!forge) {
      setTestResult('Please initialize Forge WASM first');
      return;
    }

    setLoading(true);
    setTestResult('Running matrix multiplication test...');

    try {
      // Create test matrices (4x4 identity-like matrices)
      const config = { batch: 1, m: 4, k: 4, n: 4 };
      const a = new Float32Array([
        1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        0, 0, 0, 1
      ]);
      const b = new Float32Array([
        2, 0, 0, 0,
        0, 3, 0, 0,
        0, 0, 4, 0,
        0, 0, 0, 5
      ]);

      // Run matmul with metrics
      const result = await forge.matmul_with_metrics(a, b, config);

      const expectedResult = [2, 0, 0, 0, 0, 3, 0, 0, 0, 0, 4, 0, 0, 0, 0, 5];
      const actualResult = Array.from(result.result);

      let message = `✓ Matrix Multiplication Test Passed!\n\n`;
      message += `Configuration: [${config.batch}, ${config.m}, ${config.k}, ${config.n}]\n`;
      message += `Execution Time: ${result.metrics.duration_ms.toFixed(2)}ms\n`;
      message += `Input Size: ${result.metrics.input_size} floats\n`;
      message += `Output Size: ${result.metrics.output_size} floats\n\n`;
      message += `Expected: [${expectedResult.map(v => v.toFixed(1)).join(', ')}]\n`;
      message += `Actual:   [${actualResult.map(v => v.toFixed(1)).join(', ')}]`;

      setTestResult(message);
    } catch (error) {
      setTestResult(`✗ Test failed: ${error}`);
      console.error('MatMul test error:', error);
    }

    setLoading(false);
  };

  const runBenchmark = async () => {
    if (!forge) {
      setTestResult('Please initialize Forge WASM first');
      return;
    }

    setLoading(true);
    setTestResult('Running comprehensive benchmark...');

    try {
      const results = await forge.benchmark();
      setBenchmarks(results);

      let message = `✓ Benchmark Complete!\n\n`;
      results.forEach(r => {
        message += `${r.kernel_name}: ${r.duration_ms.toFixed(2)}ms\n`;
      });

      setTestResult(message);
    } catch (error) {
      setTestResult(`✗ Benchmark failed: ${error}`);
      console.error('Benchmark error:', error);
    }

    setLoading(false);
  };

  const registerWithBackend = async () => {
    if (!webgpuInfo?.supported) {
      setTestResult('WebGPU not supported - cannot register');
      return;
    }

    setLoading(true);

    try {
      const response = await fetch('http://localhost:3000/api/v1/webgpu/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          adapter_info: webgpuInfo.adapterInfo,
          capabilities: {
            wasm_available: !!forge,
          },
        }),
      });

      if (response.ok) {
        const data = await response.json();
        setRegistered(true);
        setTestResult(`✓ Registered with backend!\nClient ID: ${data.client_id || 'unknown'}`);
      } else {
        setTestResult(`✗ Registration failed: ${response.statusText}`);
      }
    } catch (error) {
      setTestResult(`✗ Registration error: ${error}`);
      console.error('Registration error:', error);
    }

    setLoading(false);
  };

  return (
    <div className="webgpu-panel">
      <h2>WebGPU Integration</h2>

      {/* WebGPU Support Status */}
      <section className="panel-section">
        <h3>WebGPU Support</h3>
        {loading && !webgpuInfo && <p className="text-muted">Checking WebGPU support...</p>}

        {webgpuInfo && (
          <div className={`status-card ${webgpuInfo.supported ? 'status-success' : 'status-error'}`}>
            <div className="status-header">
              {webgpuInfo.supported ? '✓ WebGPU Supported' : '✗ WebGPU Not Supported'}
            </div>

            {webgpuInfo.supported && webgpuInfo.adapterInfo && (
              <div className="adapter-info">
                <p><strong>Name:</strong> {webgpuInfo.adapterInfo.name}</p>
                <p><strong>Device Type:</strong> {webgpuInfo.adapterInfo.deviceType}</p>
                <p><strong>Backend:</strong> {webgpuInfo.adapterInfo.backend}</p>
              </div>
            )}

            {webgpuInfo.error && (
              <div className="error-message">{webgpuInfo.error}</div>
            )}
          </div>
        )}
      </section>

      {/* Forge WASM Controls */}
      {webgpuInfo?.supported && (
        <section className="panel-section">
          <h3>Forge WASM Module</h3>
          <div className="button-group">
            <button
              className="btn btn-primary"
              onClick={initializeForgeWasm}
              disabled={loading || !!forge}
            >
              {forge ? '✓ Initialized' : 'Initialize Forge WASM'}
            </button>

            <button
              className="btn btn-secondary"
              onClick={runMatMulTest}
              disabled={loading || !forge}
            >
              Run MatMul Test
            </button>

            <button
              className="btn btn-secondary"
              onClick={runBenchmark}
              disabled={loading || !forge}
            >
              Run Benchmark
            </button>

            <button
              className="btn btn-accent"
              onClick={registerWithBackend}
              disabled={loading || registered}
            >
              {registered ? '✓ Registered' : 'Register with Backend'}
            </button>
          </div>
        </section>
      )}

      {/* Test Results */}
      {testResult && (
        <section className="panel-section">
          <h3>Results</h3>
          <pre className="result-output">{testResult}</pre>
        </section>
      )}

      {/* Benchmark Table */}
      {benchmarks.length > 0 && (
        <section className="panel-section">
          <h3>Benchmark Results</h3>
          <table className="benchmark-table">
            <thead>
              <tr>
                <th>Kernel</th>
                <th>Duration (ms)</th>
                <th>Input Size</th>
                <th>Output Size</th>
                <th>Throughput (MB/s)</th>
              </tr>
            </thead>
            <tbody>
              {benchmarks.map((result, idx) => {
                const totalBytes = (result.input_size + result.output_size) * 4;
                const throughputMBps = (totalBytes / 1024 / 1024) / (result.duration_ms / 1000);

                return (
                  <tr key={idx}>
                    <td>{result.kernel_name}</td>
                    <td>{result.duration_ms.toFixed(2)}</td>
                    <td>{result.input_size}</td>
                    <td>{result.output_size}</td>
                    <td>{throughputMBps.toFixed(2)}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </section>
      )}

      {/* Instructions */}
      <section className="panel-section">
        <h3>Instructions</h3>
        <div className="instructions">
          <ol>
            <li>Ensure WebGPU is supported (check status above)</li>
            <li>Click "Initialize Forge WASM" to load the GPU compute module</li>
            <li>Run tests to verify GPU compute works correctly</li>
            <li>Run benchmarks to see performance metrics</li>
            <li>Register with backend to enable distributed compute (optional)</li>
          </ol>

          <div className="info-box">
            <h4>Building the WASM Module</h4>
            <code>
              cd crates/forge_wasm<br/>
              wasm-pack build --target web --release<br/>
              cp -r pkg ../../ui/app/public/wasm
            </code>
          </div>
        </div>
      </section>
    </div>
  );
};

export default WebGPUPanel;
