'use client';

import { useDiagramStore } from '@/store/diagramStore';
import { DiagramTypeSchema } from '@diagramlab/core';
import type { DiagramType } from '@diagramlab/core';
import { Save, Moon, Sun } from 'lucide-react';
import { useState } from 'react';

export default function Header() {
  const { getCurrentDiagram, updateDiagramType, markSaved, isDirty } = useDiagramStore();
  const currentDiagram = getCurrentDiagram();
  const [darkMode, setDarkMode] = useState(false);

  const diagramTypes: DiagramType[] = DiagramTypeSchema.options;

  const handleSave = async () => {
    if (currentDiagram) {
      // TODO: API call to save diagram
      markSaved();
      console.log('Diagram saved:', currentDiagram);
    }
  };

  const toggleTheme = () => {
    setDarkMode(!darkMode);
    document.documentElement.classList.toggle('dark');
  };

  return (
    <header className="h-14 border-b border-border bg-background flex items-center justify-between px-4">
      <div className="flex items-center gap-4">
        <h1 className="text-xl font-bold">Diagramlab</h1>

        {currentDiagram && (
          <>
            <div className="h-6 w-px bg-border" />

            <input
              type="text"
              value={currentDiagram.title}
              className="px-2 py-1 text-sm border border-border rounded bg-background focus:outline-none focus:ring-2 focus:ring-primary"
              placeholder="Diagram title"
              readOnly
            />

            <select
              value={currentDiagram.diagramType}
              onChange={(e) =>
                updateDiagramType(currentDiagram.id, e.target.value as DiagramType)
              }
              className="px-2 py-1 text-sm border border-border rounded bg-background focus:outline-none focus:ring-2 focus:ring-primary"
            >
              {diagramTypes.map((type) => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </select>
          </>
        )}
      </div>

      <div className="flex items-center gap-2">
        {isDirty && (
          <span className="text-xs text-muted-foreground">Unsaved changes</span>
        )}

        <button
          onClick={handleSave}
          disabled={!isDirty}
          className="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          <Save size={16} />
          Save
        </button>

        <button
          onClick={toggleTheme}
          className="p-2 rounded hover:bg-secondary"
          aria-label="Toggle theme"
        >
          {darkMode ? <Sun size={18} /> : <Moon size={18} />}
        </button>
      </div>
    </header>
  );
}
