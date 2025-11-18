import { create } from 'zustand';
import type { ChatMessage, ChatRole } from '@diagramlab/core';

interface ChatState {
  sessionId?: string;
  messages: ChatMessage[];
  isStreaming: boolean;
  addMessage: (role: ChatRole, content: string, mermaidCode?: string) => void;
  setStreaming: (streaming: boolean) => void;
  clearMessages: () => void;
  setSessionId: (id: string) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  sessionId: undefined,
  messages: [],
  isStreaming: false,

  addMessage: (role, content, mermaidCode) =>
    set((state) => ({
      messages: [
        ...state.messages,
        {
          id: `msg-${Date.now()}`,
          sessionId: state.sessionId || '',
          role,
          content,
          mermaidCode,
          createdAt: new Date(),
        },
      ],
    })),

  setStreaming: (streaming) => set({ isStreaming: streaming }),

  clearMessages: () => set({ messages: [] }),

  setSessionId: (id) => set({ sessionId: id }),
}));
