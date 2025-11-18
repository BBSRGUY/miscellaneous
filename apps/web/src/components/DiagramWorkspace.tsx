'use client';

import { useState } from 'react';
import Header from './Header';
import CodeEditor from './CodeEditor';
import DiagramPreview from './DiagramPreview';
import AIAssistant from './AIAssistant';

export default function DiagramWorkspace() {
  const [editorWidth, setEditorWidth] = useState(33); // percentage
  const [previewWidth, setPreviewWidth] = useState(34); // percentage
  // AI panel takes remaining space

  return (
    <div className="flex flex-col h-screen">
      <Header />

      <div className="flex flex-1 overflow-hidden">
        {/* Code Editor Pane */}
        <div
          className="border-r border-border overflow-hidden"
          style={{ width: `${editorWidth}%` }}
        >
          <CodeEditor />
        </div>

        {/* Resizer 1 */}
        <div
          className="w-1 bg-border hover:bg-primary cursor-col-resize"
          onMouseDown={(e) => {
            e.preventDefault();
            const startX = e.clientX;
            const startWidth = editorWidth;

            const handleMouseMove = (e: MouseEvent) => {
              const delta = ((e.clientX - startX) / window.innerWidth) * 100;
              const newWidth = Math.max(20, Math.min(50, startWidth + delta));
              setEditorWidth(newWidth);
            };

            const handleMouseUp = () => {
              document.removeEventListener('mousemove', handleMouseMove);
              document.removeEventListener('mouseup', handleMouseUp);
            };

            document.addEventListener('mousemove', handleMouseMove);
            document.addEventListener('mouseup', handleMouseUp);
          }}
        />

        {/* Diagram Preview Pane */}
        <div
          className="border-r border-border overflow-hidden"
          style={{ width: `${previewWidth}%` }}
        >
          <DiagramPreview />
        </div>

        {/* Resizer 2 */}
        <div
          className="w-1 bg-border hover:bg-primary cursor-col-resize"
          onMouseDown={(e) => {
            e.preventDefault();
            const startX = e.clientX;
            const startWidth = previewWidth;

            const handleMouseMove = (e: MouseEvent) => {
              const delta = ((e.clientX - startX) / window.innerWidth) * 100;
              const newWidth = Math.max(20, Math.min(50, startWidth + delta));
              setPreviewWidth(newWidth);
            };

            const handleMouseUp = () => {
              document.removeEventListener('mousemove', handleMouseMove);
              document.removeEventListener('mouseup', handleMouseUp);
            };

            document.addEventListener('mousemove', handleMouseMove);
            document.addEventListener('mouseup', handleMouseUp);
          }}
        />

        {/* AI Assistant Pane */}
        <div className="flex-1 overflow-hidden">
          <AIAssistant />
        </div>
      </div>
    </div>
  );
}
