export interface LLMMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface LLMChatParams {
  system: string;
  messages: LLMMessage[];
  temperature?: number;
  maxTokens?: number;
}

export interface LLMResponse {
  responseText: string;
  mermaidCode?: string;
}

export interface LLMProvider {
  chat(params: LLMChatParams): AsyncIterable<string>;
  chatComplete(params: LLMChatParams): Promise<string>;
}

export type DiagramMode = 'generate' | 'modify' | 'explain' | 'convert';

export interface DiagramContext {
  mode: DiagramMode;
  diagramType: string;
  currentCode?: string;
  config?: Record<string, unknown>;
  instruction: string;
  targetType?: string; // For convert mode
  notes?: string;
}
