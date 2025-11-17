import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Model {
  id: string;
  name: string;
  backend: string;
  format: string;
  quantization: string;
  size_bytes: number | null;
  enabled: boolean;
}

const ModelsPanel = () => {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    setLoading(true);
    setError('');
    try {
      const result = await invoke<Model[]>('list_models');
      setModels(result);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const formatSize = (bytes: number | null): string => {
    if (!bytes) return 'N/A';
    const kb = bytes / 1024;
    const mb = kb / 1024;
    const gb = mb / 1024;

    if (gb >= 1) return `${gb.toFixed(2)} GB`;
    if (mb >= 1) return `${mb.toFixed(2)} MB`;
    if (kb >= 1) return `${kb.toFixed(2)} KB`;
    return `${bytes} B`;
  };

  return (
    <div className="panel-content">
      <div className="panel-header-row">
        <h2>Models</h2>
        <button className="btn-refresh" onClick={loadModels} disabled={loading}>
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      {error && (
        <div className="error-message">
          <strong>Error:</strong> {error}
        </div>
      )}

      {!error && models.length === 0 && !loading && (
        <div className="info-box">
          <p>No models registered yet.</p>
          <p className="hint">
            <strong>Add a model:</strong> Run <code>forge models add --name test --path ./model.gguf</code> in the terminal
          </p>
        </div>
      )}

      {models.length > 0 && (
        <div className="table-container">
          <table className="models-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Backend</th>
                <th>Format</th>
                <th>Quantization</th>
                <th>Size</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {models.map((model) => (
                <tr key={model.id}>
                  <td>
                    <strong>{model.name}</strong>
                    <br />
                    <small className="text-muted">{model.id}</small>
                  </td>
                  <td>{model.backend}</td>
                  <td>{model.format}</td>
                  <td>{model.quantization}</td>
                  <td>{formatSize(model.size_bytes)}</td>
                  <td>
                    <span className={`status-badge ${model.enabled ? 'enabled' : 'disabled'}`}>
                      {model.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default ModelsPanel;
