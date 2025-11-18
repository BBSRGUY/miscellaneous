/**
 * API Client for Forge Platform
 *
 * Centralizes all HTTP API calls with proper error handling and base URL management.
 * Works in both browser and Tauri contexts.
 */

// Determine API base URL based on environment
const getApiBaseUrl = (): string => {
  // In Tauri, use localhost
  // In browser dev mode, use the dev server proxy or localhost
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    return 'http://localhost:3000';
  }
  // For development, you can configure Vite proxy or use direct URL
  return import.meta.env.VITE_API_URL || 'http://localhost:3000';
};

const API_BASE_URL = getApiBaseUrl();

/**
 * Custom error class for API errors
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public status?: number,
    public data?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/**
 * Generic fetch wrapper with error handling
 */
async function fetchApi<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`;

  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      let errorData;
      try {
        errorData = await response.json();
      } catch {
        errorData = { error: response.statusText };
      }

      throw new ApiError(
        errorData.error || `HTTP ${response.status}: ${response.statusText}`,
        response.status,
        errorData
      );
    }

    // Handle empty responses
    const contentType = response.headers.get('content-type');
    if (!contentType || !contentType.includes('application/json')) {
      return {} as T;
    }

    return await response.json();
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }

    // Network errors or other fetch failures
    throw new ApiError(
      `Network error: ${error instanceof Error ? error.message : 'Unknown error'}`,
      undefined,
      error
    );
  }
}

// ============================================================================
// Type Definitions
// ============================================================================

export interface HealthResponse {
  status: string;
  version: string;
  uptime_seconds: number;
}

export interface ModelListItem {
  id: string;
  name: string;
  backend: string;
  format: string;
  quantization: string;
  size_bytes: number | null;
  enabled: boolean;
}

export interface RegisterModelRequest {
  name: string;
  path: string;
  backend: string;
  format: string;
  quantization?: string;
  tags?: string[];
}

export interface RegisterModelResponse {
  id: string;
  message: string;
}

export interface LoadModelRequest {
  device?: string;
}

export interface ChatRequest {
  session_id?: string;
  model_id: string;
  prompt: string;
  stream?: boolean;
  priority?: string;
}

export interface ChatResponse {
  task_id: string;
  session_id: string;
}

export interface SessionDetail {
  id: string;
  name: string;
  model_id: string | null;
  created_at: string;
  updated_at: string;
  message_count: number;
  messages: Message[];
}

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  created_at: string;
}

export interface JobListItem {
  id: string;
  state: string;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  duration_ms: number | null;
}

export interface StreamChunk {
  text?: string;
  finish_reason?: string;
  error?: string;
}

// ============================================================================
// API Methods
// ============================================================================

/**
 * Health check
 */
export async function checkHealth(): Promise<HealthResponse> {
  return fetchApi<HealthResponse>('/api/v1/health');
}

/**
 * List all models
 */
export async function listModels(): Promise<ModelListItem[]> {
  return fetchApi<ModelListItem[]>('/api/v1/models');
}

/**
 * Register a new model
 */
export async function registerModel(
  request: RegisterModelRequest
): Promise<RegisterModelResponse> {
  return fetchApi<RegisterModelResponse>('/api/v1/models', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

/**
 * Load a model into memory
 */
export async function loadModel(
  modelId: string,
  request: LoadModelRequest = {}
): Promise<{ message: string }> {
  return fetchApi<{ message: string }>(`/api/v1/models/${modelId}/load`, {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

/**
 * Submit a chat request (non-streaming)
 */
export async function submitChat(
  request: ChatRequest
): Promise<ChatResponse> {
  return fetchApi<ChatResponse>('/api/v1/chat', {
    method: 'POST',
    body: JSON.stringify({ ...request, stream: false }),
  });
}

/**
 * Stream chat responses using Server-Sent Events
 */
export async function* streamChat(
  request: ChatRequest
): AsyncGenerator<StreamChunk, void, unknown> {
  const url = `${API_BASE_URL}/api/v1/chat`;

  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ ...request, stream: true }),
  });

  if (!response.ok) {
    throw new ApiError(`HTTP ${response.status}: ${response.statusText}`, response.status);
  }

  const reader = response.body?.getReader();
  if (!reader) {
    throw new ApiError('No response body');
  }

  const decoder = new TextDecoder();
  let buffer = '';

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      // Process complete SSE events
      const events = buffer.split('\n\n');
      buffer = events.pop() || '';

      for (const event of events) {
        const lines = event.split('\n');
        for (const line of lines) {
          if (line.startsWith('data:')) {
            const data = line.substring(5).trim();
            try {
              const chunk = JSON.parse(data) as StreamChunk;
              yield chunk;
            } catch (e) {
              console.error('Failed to parse SSE data:', data, e);
            }
          }
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}

/**
 * Get session details with message history
 */
export async function getSession(sessionId: string): Promise<SessionDetail> {
  return fetchApi<SessionDetail>(`/api/v1/sessions/${sessionId}`);
}

/**
 * List all jobs
 */
export async function listJobs(): Promise<JobListItem[]> {
  return fetchApi<JobListItem[]>('/api/v1/jobs');
}

/**
 * Get job details by ID
 */
export async function getJob(jobId: string): Promise<JobListItem> {
  return fetchApi<JobListItem>(`/api/v1/jobs/${jobId}`);
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Format file size in human-readable format
 */
export function formatSize(bytes: number | null): string {
  if (!bytes) return 'N/A';
  const kb = bytes / 1024;
  const mb = kb / 1024;
  const gb = mb / 1024;

  if (gb >= 1) return `${gb.toFixed(2)} GB`;
  if (mb >= 1) return `${mb.toFixed(2)} MB`;
  if (kb >= 1) return `${kb.toFixed(2)} KB`;
  return `${bytes} B`;
}

/**
 * Format timestamp for display
 */
export function formatTimestamp(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleString();
  } catch {
    return timestamp.substring(0, 19).replace('T', ' ');
  }
}

/**
 * Format duration in human-readable format
 */
export function formatDuration(ms: number | null): string {
  if (!ms) return '-';
  if (ms < 1000) return `${ms}ms`;
  const seconds = ms / 1000;
  if (seconds < 60) return `${seconds.toFixed(2)}s`;
  const minutes = seconds / 60;
  return `${minutes.toFixed(2)}m`;
}

export default {
  checkHealth,
  listModels,
  registerModel,
  loadModel,
  submitChat,
  streamChat,
  getSession,
  listJobs,
  getJob,
  formatSize,
  formatTimestamp,
  formatDuration,
};
