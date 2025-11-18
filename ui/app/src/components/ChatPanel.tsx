import { useState, useEffect, useRef } from 'react';
import * as api from '../api/client';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  streaming?: boolean;
}

const ChatPanel = () => {
  const [models, setModels] = useState<api.ModelListItem[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [streaming, setStreaming] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadModels();
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const loadModels = async () => {
    try {
      const result = await api.listModels();
      setModels(result.filter(m => m.enabled));
      if (result.length > 0 && !selectedModel) {
        setSelectedModel(result[0].id);
      }
    } catch (err) {
      console.error('Failed to load models:', err);
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const handleSend = async () => {
    if (!input.trim() || !selectedModel || loading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: input.trim(),
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setLoading(true);
    setError('');

    try {
      if (streaming) {
        await handleStreamingChat(userMessage.content);
      } else {
        await handleNonStreamingChat(userMessage.content);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send message');
      console.error('Chat error:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleStreamingChat = async (prompt: string) => {
    const assistantMessageId = Date.now().toString();
    const assistantMessage: ChatMessage = {
      id: assistantMessageId,
      role: 'assistant',
      content: '',
      timestamp: new Date(),
      streaming: true,
    };

    setMessages(prev => [...prev, assistantMessage]);

    try {
      const stream = api.streamChat({
        model_id: selectedModel,
        prompt,
      });

      for await (const chunk of stream) {
        if (chunk.error) {
          throw new Error(chunk.error);
        }

        if (chunk.text) {
          setMessages(prev =>
            prev.map(msg =>
              msg.id === assistantMessageId
                ? { ...msg, content: msg.content + chunk.text }
                : msg
            )
          );
        }

        if (chunk.finish_reason) {
          setMessages(prev =>
            prev.map(msg =>
              msg.id === assistantMessageId
                ? { ...msg, streaming: false }
                : msg
            )
          );
          break;
        }
      }
    } catch (err) {
      // Remove the failed message
      setMessages(prev => prev.filter(msg => msg.id !== assistantMessageId));
      throw err;
    }
  };

  const handleNonStreamingChat = async (prompt: string) => {
    const response = await api.submitChat({
      model_id: selectedModel,
      prompt,
    });

    const assistantMessage: ChatMessage = {
      id: response.task_id,
      role: 'assistant',
      content: `Task submitted: ${response.task_id}\nCheck the Jobs tab for status.`,
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, assistantMessage]);
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const clearChat = () => {
    setMessages([]);
    setError('');
  };

  return (
    <div className="chat-panel">
      {/* Chat Header */}
      <div className="chat-header">
        <div className="chat-controls">
          <div className="form-group-inline">
            <label htmlFor="model-select">Model:</label>
            <select
              id="model-select"
              value={selectedModel}
              onChange={(e) => setSelectedModel(e.target.value)}
              disabled={loading}
              className="model-select"
            >
              {models.length === 0 && (
                <option value="">No models available</option>
              )}
              {models.map((model) => (
                <option key={model.id} value={model.id}>
                  {model.name} ({model.backend})
                </option>
              ))}
            </select>
          </div>

          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={streaming}
              onChange={(e) => setStreaming(e.target.checked)}
              disabled={loading}
            />
            Streaming
          </label>

          <button
            className="btn-secondary btn-small"
            onClick={clearChat}
            disabled={loading || messages.length === 0}
          >
            Clear
          </button>
        </div>

        {models.length === 0 && (
          <div className="warning-message">
            <strong>No models available.</strong> Register a model in the Models tab first.
          </div>
        )}
      </div>

      {/* Messages Area */}
      <div className="chat-messages">
        {messages.length === 0 && (
          <div className="chat-empty">
            <h3>Welcome to Forge Chat</h3>
            <p>Select a model and start chatting!</p>
            <div className="chat-features">
              <h4>Features:</h4>
              <ul>
                <li>Real-time streaming responses</li>
                <li>Multiple model support</li>
                <li>Session management (coming soon)</li>
                <li>Message history</li>
              </ul>
            </div>
          </div>
        )}

        {messages.map((message) => (
          <div key={message.id} className={`message message-${message.role}`}>
            <div className="message-header">
              <strong className="message-role">
                {message.role === 'user' ? 'You' : message.role === 'assistant' ? 'Assistant' : 'System'}
              </strong>
              <span className="message-time">
                {message.timestamp.toLocaleTimeString()}
              </span>
            </div>
            <div className="message-content">
              {message.content}
              {message.streaming && (
                <span className="streaming-cursor">▊</span>
              )}
            </div>
          </div>
        ))}

        {error && (
          <div className="message message-error">
            <div className="message-content">
              <strong>Error:</strong> {error}
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input Area */}
      <div className="chat-input-container">
        <textarea
          className="chat-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder={
            selectedModel
              ? "Type your message... (Shift+Enter for new line, Enter to send)"
              : "Select a model to start chatting"
          }
          disabled={loading || !selectedModel}
          rows={3}
        />
        <button
          className="btn-primary btn-send"
          onClick={handleSend}
          disabled={loading || !selectedModel || !input.trim()}
        >
          {loading ? 'Sending...' : 'Send'}
        </button>
      </div>
    </div>
  );
};

export default ChatPanel;
