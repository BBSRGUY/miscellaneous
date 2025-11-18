import { MermaidConfig } from './types';
import type { DiagramType } from '@diagramlab/core';

// Default global configuration
export const getDefaultConfig = (): MermaidConfig => ({
  theme: 'default',
  look: 'classic',
  logLevel: 'error',
  securityLevel: 'strict',
  startOnLoad: false,
  arrowMarkerAbsolute: false,
  fontFamily: '"Open Sans", sans-serif',
  fontSize: 16,
  darkMode: false,
  htmlLabels: true,
  wrap: false,
  suppressErrorRendering: false,

  // Default flowchart config
  flowchart: {
    htmlLabels: true,
    curve: 'basis',
    useMaxWidth: true,
    nodeSpacing: 50,
    rankSpacing: 50,
    padding: 15,
  },

  // Default sequence config
  sequence: {
    mirrorActors: true,
    useMaxWidth: true,
    rightAngles: false,
    showSequenceNumbers: false,
    actorFontSize: 14,
    messageFontSize: 16,
    wrap: false,
  },

  // Default class config
  class: {
    useMaxWidth: true,
    padding: 5,
  },

  // Default state config
  state: {
    useMaxWidth: true,
  },

  // Default ER config
  er: {
    useMaxWidth: true,
    layoutDirection: 'TB',
  },

  // Default gantt config
  gantt: {
    useMaxWidth: true,
    displayMode: 'compact',
    topAxis: false,
  },

  // Default journey config
  journey: {
    useMaxWidth: true,
  },

  // Default pie config
  pie: {
    useMaxWidth: true,
    textPosition: 0.75,
  },

  // Default quadrant config
  quadrantChart: {
    chartWidth: 500,
    chartHeight: 500,
    pointRadius: 5,
  },

  // Default XY chart config
  xyChart: {
    width: 700,
    height: 500,
    chartOrientation: 'vertical',
  },

  // Default requirement config
  requirement: {
    useMaxWidth: true,
  },

  // Default architecture config
  architecture: {
    useMaxWidth: true,
    iconSize: 40,
  },

  // Default mindmap config
  mindmap: {
    useMaxWidth: true,
    maxNodeWidth: 200,
  },

  // Default kanban config
  kanban: {
    useMaxWidth: true,
  },

  // Default gitGraph config
  gitGraph: {
    useMaxWidth: true,
    showBranches: true,
    showCommitLabel: true,
  },

  // Default C4 config
  c4: {
    useMaxWidth: true,
    c4ShapeInRow: 4,
    c4BoundaryInRow: 2,
  },

  // Default sankey config
  sankey: {
    useMaxWidth: true,
    showValues: true,
    linkColor: 'gradient',
  },

  // Default packet config
  packet: {
    useMaxWidth: true,
    showBits: true,
    bitsPerRow: 32,
  },

  // Default block config
  block: {
    useMaxWidth: true,
    padding: 8,
  },

  // Default radar config
  radar: {
    useMaxWidth: true,
  },
});

// Get default config for a specific diagram type
export const getDefaultConfigForType = (type: DiagramType): Partial<MermaidConfig> => {
  const baseConfig = getDefaultConfig();

  const typeSpecificOverrides: Record<DiagramType, Partial<MermaidConfig>> = {
    flowchart: { flowchart: baseConfig.flowchart },
    sequence: { sequence: baseConfig.sequence },
    class: { class: baseConfig.class },
    state: { state: baseConfig.state },
    er: { er: baseConfig.er },
    journey: { journey: baseConfig.journey },
    gantt: { gantt: baseConfig.gantt },
    pie: { pie: baseConfig.pie },
    quadrantChart: { quadrantChart: baseConfig.quadrantChart },
    gitGraph: { gitGraph: baseConfig.gitGraph },
    c4: { c4: baseConfig.c4 },
    mindmap: { mindmap: baseConfig.mindmap },
    timeline: { timeline: baseConfig.timeline },
    sankey: { sankey: baseConfig.sankey },
    xyChart: { xyChart: baseConfig.xyChart },
    block: { block: baseConfig.block },
    packet: { packet: baseConfig.packet },
    kanban: { kanban: baseConfig.kanban },
    architecture: { architecture: baseConfig.architecture },
    radar: { radar: baseConfig.radar },
    treemap: {}, // Treemap doesn't have specific config yet
    requirement: { requirement: baseConfig.requirement },
  };

  return {
    ...baseConfig,
    ...typeSpecificOverrides[type],
  };
};

// Merge configs with validation
export const mergeConfig = (
  base: MermaidConfig,
  overrides: Partial<MermaidConfig>,
): MermaidConfig => {
  return {
    ...base,
    ...overrides,
    // Deep merge for nested configs
    flowchart: { ...base.flowchart, ...overrides.flowchart },
    sequence: { ...base.sequence, ...overrides.sequence },
    class: { ...base.class, ...overrides.class },
    state: { ...base.state, ...overrides.state },
    er: { ...base.er, ...overrides.er },
    gantt: { ...base.gantt, ...overrides.gantt },
    journey: { ...base.journey, ...overrides.journey },
    timeline: { ...base.timeline, ...overrides.timeline },
    pie: { ...base.pie, ...overrides.pie },
    quadrantChart: { ...base.quadrantChart, ...overrides.quadrantChart },
    xyChart: { ...base.xyChart, ...overrides.xyChart },
    requirement: { ...base.requirement, ...overrides.requirement },
    architecture: { ...base.architecture, ...overrides.architecture },
    mindmap: { ...base.mindmap, ...overrides.mindmap },
    kanban: { ...base.kanban, ...overrides.kanban },
    gitGraph: { ...base.gitGraph, ...overrides.gitGraph },
    c4: { ...base.c4, ...overrides.c4 },
    sankey: { ...base.sankey, ...overrides.sankey },
    packet: { ...base.packet, ...overrides.packet },
    block: { ...base.block, ...overrides.block },
    radar: { ...base.radar, ...overrides.radar },
  };
};
