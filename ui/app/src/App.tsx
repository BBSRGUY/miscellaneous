import { useState, useEffect } from 'react';
import * as api from './api/client';
import Terminal from './components/Terminal';
import ChatPanel from './components/ChatPanel';
import ModelsPanel from './components/ModelsPanel';
import JobsPanel from './components/JobsPanel';
import WebGPUPanel from './components/WebGPUPanel';
import './App.css';

function App() {
  const [activeTab, setActiveTab] = useState<'chat' | 'models' | 'jobs' | 'webgpu'>('chat');
  const [apiStatus, setApiStatus] = useState<'checking' | 'healthy' | 'error'>('checking');
  const [apiVersion, setApiVersion] = useState<string>('');

  useEffect(() => {
    // Check API health on startup
    checkHealth();
    // Set up periodic health checks
    const interval = setInterval(checkHealth, 10000);
    return () => clearInterval(interval);
  }, []);

  const checkHealth = async () => {
    try {
      const health = await api.checkHealth();
      setApiStatus('healthy');
      setApiVersion(health.version);
    } catch (error) {
      console.error('API health check failed:', error);
      setApiStatus('error');
    }
  };

  return (
    <div className="app-container">
      {/* Top Navigation Bar */}
      <nav className="top-nav">
        <div className="nav-left">
          <h1 className="app-title">Forge - Local LLM Platform</h1>
          <div className={`status-indicator status-${apiStatus}`}>
            <span className="status-dot"></span>
            {apiStatus === 'healthy' ? `API v${apiVersion}` :
             apiStatus === 'checking' ? 'Checking...' :
             'API Offline'}
          </div>
        </div>
        <div className="nav-right">
          {apiStatus === 'error' && (
            <div className="error-hint">
              <span>Start the daemon: <code>forge serve</code></span>
            </div>
          )}
        </div>
      </nav>

      {/* Main Content: Split Layout */}
      <div className="main-content">
        {/* Left Panel: Terminal */}
        <div className="left-panel">
          <div className="panel-header">
            <h3>Forge CLI Terminal</h3>
          </div>
          <Terminal />
        </div>

        {/* Right Panel: Web UI with Tabs */}
        <div className="right-panel">
          <div className="tab-bar">
            <button
              className={`tab ${activeTab === 'chat' ? 'active' : ''}`}
              onClick={() => setActiveTab('chat')}
            >
              Chat
            </button>
            <button
              className={`tab ${activeTab === 'models' ? 'active' : ''}`}
              onClick={() => setActiveTab('models')}
            >
              Models
            </button>
            <button
              className={`tab ${activeTab === 'jobs' ? 'active' : ''}`}
              onClick={() => setActiveTab('jobs')}
            >
              Jobs
            </button>
            <button
              className={`tab ${activeTab === 'webgpu' ? 'active' : ''}`}
              onClick={() => setActiveTab('webgpu')}
            >
              WebGPU
            </button>
          </div>

          <div className="tab-content">
            {activeTab === 'chat' && <ChatPanel />}
            {activeTab === 'models' && <ModelsPanel />}
            {activeTab === 'jobs' && <JobsPanel />}
            {activeTab === 'webgpu' && <WebGPUPanel />}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
