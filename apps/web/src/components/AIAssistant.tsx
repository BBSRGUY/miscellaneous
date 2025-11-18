'use client';

import { useChatStore } from '@/store/chatStore';
import { useDiagramStore } from '@/store/diagramStore';
import { Send, Sparkles } from 'lucide-react';
import { useState } from 'react';

export default function AIAssistant() {
  const { messages, addMessage, isStreaming } = useChatStore();
  const { getCurrentDiagram, updateCode } = useDiagramStore();
  const [input, setInput] = useState('');

  const handleSend = async () => {
    if (!input.trim() || isStreaming) return;

    const userMessage = input.trim();
    setInput('');
    addMessage('user', userMessage);

    // TODO: Implement actual AI integration
    // For now, just echo back
    setTimeout(() => {
      addMessage(
        'assistant',
        `I received your message: "${userMessage}". AI integration is coming soon!`,
      );
    }, 500);
  };

  const handleApplyCode = (code: string) => {
    const currentDiagram = getCurrentDiagram();
    if (currentDiagram && code) {
      updateCode(currentDiagram.id, code);
    }
  };

  const quickActions = [
    { label: 'Generate from description', icon: Sparkles },
    { label: 'Refine layout', icon: Sparkles },
    { label: 'Explain diagram', icon: Sparkles },
  ];

  return (
    <div className="flex flex-col h-full">
      <div className="h-10 border-b border-border flex items-center px-4 bg-muted">
        <h2 className="text-sm font-semibold">AI Assistant</h2>
      </div>

      {/* Quick Actions */}
      <div className="border-b border-border p-3 bg-secondary/50">
        <div className="grid grid-cols-1 gap-2">
          {quickActions.map((action) => (
            <button
              key={action.label}
              className="px-3 py-2 text-xs border border-border rounded hover:bg-background flex items-center gap-2"
            >
              <action.icon size={14} />
              {action.label}
            </button>
          ))}
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 ? (
          <div className="text-center text-muted-foreground text-sm mt-8">
            <Sparkles size={32} className="mx-auto mb-2 opacity-50" />
            <p>Ask me to generate, modify, or explain diagrams</p>
          </div>
        ) : (
          messages.map((msg) => (
            <div
              key={msg.id}
              className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-[80%] rounded-lg px-3 py-2 ${
                  msg.role === 'user'
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-secondary text-foreground'
                }`}
              >
                <p className="text-sm">{msg.content}</p>

                {msg.mermaidCode && (
                  <div className="mt-2 pt-2 border-t border-border/20">
                    <pre className="text-xs bg-background/50 rounded p-2 overflow-x-auto">
                      {msg.mermaidCode}
                    </pre>
                    <button
                      onClick={() => handleApplyCode(msg.mermaidCode!)}
                      className="mt-2 px-2 py-1 text-xs bg-primary text-primary-foreground rounded hover:opacity-90"
                    >
                      Apply to Editor
                    </button>
                  </div>
                )}
              </div>
            </div>
          ))
        )}

        {isStreaming && (
          <div className="flex justify-start">
            <div className="bg-secondary rounded-lg px-3 py-2">
              <div className="flex gap-1">
                <div className="w-2 h-2 bg-foreground/50 rounded-full animate-bounce" />
                <div
                  className="w-2 h-2 bg-foreground/50 rounded-full animate-bounce"
                  style={{ animationDelay: '0.1s' }}
                />
                <div
                  className="w-2 h-2 bg-foreground/50 rounded-full animate-bounce"
                  style={{ animationDelay: '0.2s' }}
                />
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Input */}
      <div className="border-t border-border p-4">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSend()}
            placeholder="Ask AI to generate or modify diagram..."
            className="flex-1 px-3 py-2 text-sm border border-border rounded bg-background focus:outline-none focus:ring-2 focus:ring-primary"
            disabled={isStreaming}
          />
          <button
            onClick={handleSend}
            disabled={!input.trim() || isStreaming}
            className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Send size={18} />
          </button>
        </div>
      </div>
    </div>
  );
}
