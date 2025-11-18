'use client';

import { useDiagramStore } from '@/store/diagramStore';
import Editor from '@monaco-editor/react';
import { useEffect, useState } from 'react';

export default function CodeEditor() {
  const { getCurrentDiagram, updateCode } = useDiagramStore();
  const currentDiagram = getCurrentDiagram();
  const [code, setCode] = useState(currentDiagram?.mermaidCode || '');

  useEffect(() => {
    if (currentDiagram) {
      setCode(currentDiagram.mermaidCode);
    }
  }, [currentDiagram?.id]);

  const handleEditorChange = (value: string | undefined) => {
    if (value !== undefined && currentDiagram) {
      setCode(value);
      // Debounced update
      const timer = setTimeout(() => {
        updateCode(currentDiagram.id, value);
      }, 500);
      return () => clearTimeout(timer);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="h-10 border-b border-border flex items-center px-4 bg-muted">
        <h2 className="text-sm font-semibold">Code Editor</h2>
      </div>

      <div className="flex-1">
        <Editor
          height="100%"
          defaultLanguage="mermaid"
          language="plaintext"
          value={code}
          onChange={handleEditorChange}
          theme="vs-light"
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            lineNumbers: 'on',
            wordWrap: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
          }}
        />
      </div>
    </div>
  );
}
