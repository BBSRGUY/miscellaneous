import { useState, useEffect } from 'react';
import * as api from '../api/client';

const JobsPanel = () => {
  const [jobs, setJobs] = useState<api.JobListItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [autoRefresh, setAutoRefresh] = useState(true);

  useEffect(() => {
    loadJobs();

    if (autoRefresh) {
      const interval = setInterval(loadJobs, 5000);
      return () => clearInterval(interval);
    }
  }, [autoRefresh]);

  const loadJobs = async () => {
    setLoading(true);
    setError('');
    try {
      const result = await api.listJobs();
      setJobs(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load jobs');
    } finally {
      setLoading(false);
    }
  };

  const getStateColor = (state: string): string => {
    const stateLower = state.toLowerCase();
    if (stateLower.includes('pending')) return 'state-pending';
    if (stateLower.includes('running')) return 'state-running';
    if (stateLower.includes('completed') || stateLower.includes('success')) return 'state-completed';
    if (stateLower.includes('failed') || stateLower.includes('error')) return 'state-failed';
    return '';
  };

  return (
    <div className="panel-content">
      <div className="panel-header-row">
        <h2>Jobs</h2>
        <div className="button-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            Auto-refresh (5s)
          </label>
          <button className="btn-secondary" onClick={loadJobs} disabled={loading}>
            {loading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
      </div>

      {error && (
        <div className="error-message">
          <strong>Error:</strong> {error}
        </div>
      )}

      {!error && jobs.length === 0 && !loading && (
        <div className="info-box">
          <p>No active jobs.</p>
          <p className="hint">
            <strong>Create a job:</strong> Submit a chat request or start an inference task.
            Jobs will appear here for monitoring.
          </p>
        </div>
      )}

      {jobs.length > 0 && (
        <div className="table-container">
          <table className="jobs-table">
            <thead>
              <tr>
                <th>Job ID</th>
                <th>State</th>
                <th>Created</th>
                <th>Started</th>
                <th>Completed</th>
                <th>Duration</th>
              </tr>
            </thead>
            <tbody>
              {jobs.map((job) => (
                <tr key={job.id}>
                  <td>
                    <code className="job-id" title={job.id}>
                      {job.id.substring(0, 8)}...
                    </code>
                  </td>
                  <td>
                    <span className={`state-badge ${getStateColor(job.state)}`}>
                      {job.state}
                    </span>
                  </td>
                  <td>{api.formatTimestamp(job.created_at)}</td>
                  <td>{job.started_at ? api.formatTimestamp(job.started_at) : '-'}</td>
                  <td>
                    {job.completed_at ? api.formatTimestamp(job.completed_at) : '-'}
                  </td>
                  <td>{api.formatDuration(job.duration_ms)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {jobs.length > 0 && (
        <div className="jobs-summary">
          <p className="text-muted">
            Showing {jobs.length} job{jobs.length !== 1 ? 's' : ''}
            {autoRefresh && ' • Auto-refreshing every 5 seconds'}
          </p>
        </div>
      )}
    </div>
  );
};

export default JobsPanel;
