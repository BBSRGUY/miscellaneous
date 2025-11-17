import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Job {
  id: string;
  state: string;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  duration_ms: number | null;
}

const JobsPanel = () => {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    loadJobs();
    // Auto-refresh every 5 seconds
    const interval = setInterval(loadJobs, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadJobs = async () => {
    setLoading(true);
    setError('');
    try {
      const result = await invoke<Job[]>('list_jobs');
      setJobs(result);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const formatTimestamp = (timestamp: string): string => {
    return timestamp.substring(0, 19).replace('T', ' ');
  };

  const formatDuration = (ms: number | null): string => {
    if (!ms) return '-';
    if (ms < 1000) return `${ms}ms`;
    const seconds = ms / 1000;
    if (seconds < 60) return `${seconds.toFixed(2)}s`;
    const minutes = seconds / 60;
    return `${minutes.toFixed(2)}m`;
  };

  return (
    <div className="panel-content">
      <div className="panel-header-row">
        <h2>Jobs</h2>
        <button className="btn-refresh" onClick={loadJobs} disabled={loading}>
          {loading ? 'Loading...' : 'Refresh'}
        </button>
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
            <strong>Create a job:</strong> Start a chat or submit an inference request
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
                <th>Duration</th>
              </tr>
            </thead>
            <tbody>
              {jobs.map((job) => (
                <tr key={job.id}>
                  <td>
                    <code className="job-id">{job.id.substring(0, 8)}...</code>
                  </td>
                  <td>
                    <span className={`state-badge state-${job.state.toLowerCase()}`}>
                      {job.state}
                    </span>
                  </td>
                  <td>{formatTimestamp(job.created_at)}</td>
                  <td>{job.started_at ? formatTimestamp(job.started_at) : '-'}</td>
                  <td>{formatDuration(job.duration_ms)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default JobsPanel;
