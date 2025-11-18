import { useState, useEffect } from 'react';
import * as api from '../api/client';

const ModelsPanel = () => {
  const [models, setModels] = useState<api.ModelListItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [showAddForm, setShowAddForm] = useState(false);
  const [loadingModelId, setLoadingModelId] = useState<string | null>(null);

  // Form state
  const [formData, setFormData] = useState({
    name: '',
    path: '',
    backend: 'echo',
    format: 'gguf',
    quantization: '',
  });
  const [formError, setFormError] = useState('');
  const [formSubmitting, setFormSubmitting] = useState(false);

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    setLoading(true);
    setError('');
    try {
      const result = await api.listModels();
      setModels(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load models');
    } finally {
      setLoading(false);
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value,
    });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError('');
    setFormSubmitting(true);

    try {
      const request: api.RegisterModelRequest = {
        name: formData.name,
        path: formData.path,
        backend: formData.backend,
        format: formData.format,
      };

      if (formData.quantization) {
        request.quantization = formData.quantization;
      }

      await api.registerModel(request);

      // Reset form and reload models
      setFormData({
        name: '',
        path: '',
        backend: 'echo',
        format: 'gguf',
        quantization: '',
      });
      setShowAddForm(false);
      await loadModels();
    } catch (err) {
      setFormError(err instanceof Error ? err.message : 'Failed to register model');
    } finally {
      setFormSubmitting(false);
    }
  };

  const handleLoadModel = async (modelId: string) => {
    setLoadingModelId(modelId);
    try {
      await api.loadModel(modelId, { device: 'cpu' });
      // Optionally show success message
      alert('Model loaded successfully');
    } catch (err) {
      alert(`Failed to load model: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoadingModelId(null);
    }
  };

  return (
    <div className="panel-content">
      <div className="panel-header-row">
        <h2>Models</h2>
        <div className="button-group">
          <button className="btn-secondary" onClick={loadModels} disabled={loading}>
            {loading ? 'Loading...' : 'Refresh'}
          </button>
          <button className="btn-primary" onClick={() => setShowAddForm(!showAddForm)}>
            {showAddForm ? 'Cancel' : 'Add Model'}
          </button>
        </div>
      </div>

      {error && (
        <div className="error-message">
          <strong>Error:</strong> {error}
        </div>
      )}

      {/* Add Model Form */}
      {showAddForm && (
        <div className="form-container">
          <h3>Register New Model</h3>
          <form onSubmit={handleSubmit}>
            <div className="form-group">
              <label htmlFor="name">Model Name *</label>
              <input
                type="text"
                id="name"
                name="name"
                value={formData.name}
                onChange={handleInputChange}
                required
                placeholder="e.g., llama-3-8b"
              />
            </div>

            <div className="form-group">
              <label htmlFor="path">Model Path *</label>
              <input
                type="text"
                id="path"
                name="path"
                value={formData.path}
                onChange={handleInputChange}
                required
                placeholder="/path/to/model.gguf"
              />
            </div>

            <div className="form-row">
              <div className="form-group">
                <label htmlFor="backend">Backend</label>
                <select
                  id="backend"
                  name="backend"
                  value={formData.backend}
                  onChange={handleInputChange}
                >
                  <option value="echo">Echo (Testing)</option>
                  <option value="llama">Llama</option>
                  <option value="candle">Candle</option>
                </select>
              </div>

              <div className="form-group">
                <label htmlFor="format">Format</label>
                <select
                  id="format"
                  name="format"
                  value={formData.format}
                  onChange={handleInputChange}
                >
                  <option value="gguf">GGUF</option>
                  <option value="safetensors">SafeTensors</option>
                  <option value="pytorch">PyTorch</option>
                </select>
              </div>
            </div>

            <div className="form-group">
              <label htmlFor="quantization">Quantization (optional)</label>
              <input
                type="text"
                id="quantization"
                name="quantization"
                value={formData.quantization}
                onChange={handleInputChange}
                placeholder="e.g., q4_k_m, q8_0"
              />
            </div>

            {formError && (
              <div className="error-message">
                <strong>Error:</strong> {formError}
              </div>
            )}

            <div className="form-actions">
              <button
                type="button"
                className="btn-secondary"
                onClick={() => setShowAddForm(false)}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="btn-primary"
                disabled={formSubmitting}
              >
                {formSubmitting ? 'Registering...' : 'Register Model'}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Models List */}
      {!error && models.length === 0 && !loading && (
        <div className="info-box">
          <p>No models registered yet.</p>
          <p className="hint">
            <strong>Add a model:</strong> Click the "Add Model" button above or run{' '}
            <code>forge models add --name test --path ./model.gguf</code> in the terminal
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
                <th>Actions</th>
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
                  <td>{api.formatSize(model.size_bytes)}</td>
                  <td>
                    <span className={`status-badge ${model.enabled ? 'enabled' : 'disabled'}`}>
                      {model.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </td>
                  <td>
                    <button
                      className="btn-small"
                      onClick={() => handleLoadModel(model.id)}
                      disabled={loadingModelId === model.id}
                    >
                      {loadingModelId === model.id ? 'Loading...' : 'Load'}
                    </button>
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
