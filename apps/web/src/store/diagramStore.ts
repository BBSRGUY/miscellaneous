import { create } from 'zustand';
import type { DiagramType } from '@diagramlab/core';
import type { MermaidConfig } from '@diagramlab/mermaid-config';
import { getDefaultConfig } from '@diagramlab/mermaid-config';

export interface Diagram {
  id: string;
  title: string;
  diagramType: DiagramType;
  mermaidCode: string;
  mermaidConfig: MermaidConfig;
  isDirty?: boolean;
}

interface DiagramStore {
  currentProjectId?: string;
  currentDiagramId?: string;
  diagrams: Record<string, Diagram>;

  setCurrentDiagram: (id: string) => void;
  createDiagram: (diagram: Diagram) => void;
  updateCode: (id: string, code: string) => void;
  updateConfig: (id: string, config: Partial<MermaidConfig>) => void;
  updateDiagramType: (id: string, type: DiagramType) => void;
  markSaved: (id: string) => void;
  getCurrentDiagram: () => Diagram | undefined;
}

const SAMPLE_FLOWCHART = `flowchart TD
    Start([Start]) --> Input[/Enter Data/]
    Input --> Process[Process Data]
    Process --> Decision{Valid?}
    Decision -->|Yes| Output[/Display Result/]
    Decision -->|No| Error[Show Error]
    Error --> Input
    Output --> End([End])`;

export const useDiagramStore = create<DiagramStore>((set, get) => ({
  currentProjectId: 'sample-project',
  currentDiagramId: 'sample-diagram',
  diagrams: {
    'sample-diagram': {
      id: 'sample-diagram',
      title: 'Sample Flowchart',
      diagramType: 'flowchart',
      mermaidCode: SAMPLE_FLOWCHART,
      mermaidConfig: getDefaultConfig(),
      isDirty: false,
    },
  },

  setCurrentDiagram: (id) => set({ currentDiagramId: id }),

  createDiagram: (diagram) =>
    set((state) => ({
      diagrams: {
        ...state.diagrams,
        [diagram.id]: diagram,
      },
      currentDiagramId: diagram.id,
    })),

  updateCode: (id, code) =>
    set((state) => ({
      diagrams: {
        ...state.diagrams,
        [id]: {
          ...state.diagrams[id],
          mermaidCode: code,
          isDirty: true,
        },
      },
    })),

  updateConfig: (id, config) =>
    set((state) => ({
      diagrams: {
        ...state.diagrams,
        [id]: {
          ...state.diagrams[id],
          mermaidConfig: {
            ...state.diagrams[id].mermaidConfig,
            ...config,
          },
          isDirty: true,
        },
      },
    })),

  updateDiagramType: (id, type) =>
    set((state) => ({
      diagrams: {
        ...state.diagrams,
        [id]: {
          ...state.diagrams[id],
          diagramType: type,
          isDirty: true,
        },
      },
    })),

  markSaved: (id) =>
    set((state) => ({
      diagrams: {
        ...state.diagrams,
        [id]: {
          ...state.diagrams[id],
          isDirty: false,
        },
      },
    })),

  getCurrentDiagram: () => {
    const state = get();
    return state.currentDiagramId ? state.diagrams[state.currentDiagramId] : undefined;
  },
}));
