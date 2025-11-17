const ChatPanel = () => {
  return (
    <div className="panel-content">
      <h2>Chat</h2>
      <p>Interactive chat interface (coming soon)</p>
      <div className="info-box">
        <h3>Features:</h3>
        <ul>
          <li>Select model from registered models</li>
          <li>Real-time streaming responses</li>
          <li>Conversation history</li>
          <li>Session management</li>
        </ul>
        <p className="hint">
          <strong>Try the CLI:</strong> Run <code>forge chat --model &lt;model-id&gt; --stream</code> in the terminal
        </p>
      </div>
    </div>
  );
};

export default ChatPanel;
