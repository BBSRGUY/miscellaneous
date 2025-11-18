// Mermaid Configuration Types
// Based on comprehensive Mermaid config schema

export type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'fatal';
export type SecurityLevel = 'strict' | 'loose' | 'antiscript' | 'sandbox';
export type Look = 'classic' | 'handDrawn';

// Flowchart Configuration
export interface FlowchartConfig {
  titleTopMargin?: number;
  arrowMarkerAbsolute?: boolean;
  diagramPadding?: number;
  htmlLabels?: boolean;
  nodeSpacing?: number;
  rankSpacing?: number;
  curve?: string;
  padding?: number;
  useMaxWidth?: boolean;
  defaultRenderer?: string;
  wrappingWidth?: number;
}

// Sequence Diagram Configuration
export interface SequenceConfig {
  activationWidth?: number;
  diagramMarginX?: number;
  diagramMarginY?: number;
  actorMargin?: number;
  width?: number;
  height?: number;
  boxMargin?: number;
  boxTextMargin?: number;
  noteMargin?: number;
  messageMargin?: number;
  messageAlign?: string;
  mirrorActors?: boolean;
  forceMenus?: boolean;
  bottomMarginAdj?: number;
  useMaxWidth?: boolean;
  rightAngles?: boolean;
  showSequenceNumbers?: boolean;
  actorFontSize?: number;
  actorFontFamily?: string;
  actorFontWeight?: number;
  noteFontSize?: number;
  noteFontFamily?: string;
  noteFontWeight?: number;
  noteAlign?: string;
  messageFontSize?: number;
  messageFontFamily?: string;
  messageFontWeight?: number;
  wrap?: boolean;
  wrapPadding?: number;
}

// Class Diagram Configuration
export interface ClassConfig {
  titleTopMargin?: number;
  arrowMarkerAbsolute?: boolean;
  dividerMargin?: number;
  padding?: number;
  textHeight?: number;
  nodeSpacing?: number;
  rankSpacing?: number;
  defaultRenderer?: string;
  useMaxWidth?: boolean;
}

// State Diagram Configuration
export interface StateConfig {
  titleTopMargin?: number;
  arrowMarkerAbsolute?: boolean;
  dividerMargin?: number;
  sizeUnit?: number;
  padding?: number;
  textHeight?: number;
  titleShift?: number;
  noteMargin?: number;
  forkWidth?: number;
  forkHeight?: number;
  miniPadding?: number;
  fontSizeFactor?: number;
  fontSize?: number;
  labelHeight?: number;
  edgeLengthFactor?: string;
  compositeTitleSize?: number;
  radius?: number;
  useMaxWidth?: boolean;
  defaultRenderer?: string;
}

// Entity Relationship Diagram Configuration
export interface ERConfig {
  titleTopMargin?: number;
  diagramPadding?: number;
  layoutDirection?: string;
  minEntityWidth?: number;
  minEntityHeight?: number;
  entityPadding?: number;
  stroke?: string;
  fill?: string;
  fontSize?: number;
  useMaxWidth?: boolean;
}

// Gantt Diagram Configuration
export interface GanttConfig {
  titleTopMargin?: number;
  barHeight?: number;
  barGap?: number;
  topPadding?: number;
  rightPadding?: number;
  leftPadding?: number;
  gridLineStartPadding?: number;
  fontSize?: number;
  sectionFontSize?: number;
  numberSectionStyles?: number;
  axisFormat?: string;
  tickInterval?: string;
  topAxis?: boolean;
  displayMode?: string;
  useMaxWidth?: boolean;
  useWidth?: number;
}

// User Journey Configuration
export interface JourneyConfig {
  diagramMarginX?: number;
  diagramMarginY?: number;
  actorMargin?: number;
  width?: number;
  height?: number;
  boxMargin?: number;
  boxTextMargin?: number;
  noteMargin?: number;
  messageMargin?: number;
  messageAlign?: string;
  bottomMarginAdj?: number;
  useMaxWidth?: boolean;
  rightAngles?: boolean;
  taskFontSize?: number;
  taskFontFamily?: string;
  taskMargin?: number;
  activationWidth?: number;
  textPlacement?: string;
  actorColours?: string[];
  sectionFills?: string[];
  sectionColours?: string[];
}

// Timeline Configuration
export interface TimelineConfig {
  diagramMarginX?: number;
  diagramMarginY?: number;
  actorMargin?: number;
  width?: number;
  height?: number;
  boxMargin?: number;
  boxTextMargin?: number;
  noteMargin?: number;
  messageMargin?: number;
  messageAlign?: string;
  bottomMarginAdj?: number;
  useMaxWidth?: boolean;
  rightAngles?: boolean;
  disableMulticolor?: boolean;
}

// Pie Chart Configuration
export interface PieConfig {
  useWidth?: number;
  useMaxWidth?: boolean;
  textPosition?: number;
}

// Quadrant Chart Configuration
export interface QuadrantChartConfig {
  chartWidth?: number;
  chartHeight?: number;
  titleFontSize?: number;
  titlePadding?: number;
  quadrantPadding?: number;
  xAxisLabelPadding?: number;
  yAxisLabelPadding?: number;
  xAxisLabelFontSize?: number;
  yAxisLabelFontSize?: number;
  quadrantLabelFontSize?: number;
  quadrantTextTopPadding?: number;
  pointTextPadding?: number;
  pointLabelFontSize?: number;
  pointRadius?: number;
  xAxisPosition?: string;
  yAxisPosition?: string;
  quadrantInternalBorderStrokeWidth?: number;
  quadrantExternalBorderStrokeWidth?: number;
}

// XY Chart Configuration
export interface XYChartConfig {
  width?: number;
  height?: number;
  titleFontSize?: number;
  titlePadding?: number;
  showTitle?: boolean;
  xAxis?: {
    showLabel?: boolean;
    labelFontSize?: number;
    labelPadding?: number;
    showTitle?: boolean;
    titleFontSize?: number;
    titlePadding?: number;
    showTick?: boolean;
    tickLength?: number;
    tickWidth?: number;
    showAxisLine?: boolean;
    axisLineWidth?: number;
  };
  yAxis?: {
    showLabel?: boolean;
    labelFontSize?: number;
    labelPadding?: number;
    showTitle?: boolean;
    titleFontSize?: number;
    titlePadding?: number;
    showTick?: boolean;
    tickLength?: number;
    tickWidth?: number;
    showAxisLine?: boolean;
    axisLineWidth?: number;
  };
  chartOrientation?: string;
  plotReservedSpacePercent?: number;
}

// Requirement Diagram Configuration
export interface RequirementConfig {
  useWidth?: number;
  useMaxWidth?: boolean;
  rect_fill?: string;
  text_color?: string;
  rect_border_size?: string;
  rect_border_color?: string;
  rect_min_width?: number;
  rect_min_height?: number;
  fontSize?: number;
  rect_padding?: number;
  line_height?: number;
}

// Architecture Diagram Configuration
export interface ArchitectureConfig {
  diagramPadding?: number;
  iconSize?: number;
  fontSize?: number;
  useMaxWidth?: boolean;
}

// Mindmap Configuration
export interface MindmapConfig {
  padding?: number;
  maxNodeWidth?: number;
  useMaxWidth?: boolean;
}

// Kanban Configuration
export interface KanbanConfig {
  padding?: number;
  useMaxWidth?: boolean;
  ticketBaseUrl?: string;
}

// GitGraph Configuration
export interface GitGraphConfig {
  titleTopMargin?: number;
  diagramPadding?: number;
  nodeLabel?: {
    width?: number;
    height?: number;
    x?: number;
    y?: number;
  };
  mainBranchName?: string;
  mainBranchOrder?: number;
  showCommitLabel?: boolean;
  showBranches?: boolean;
  rotateCommitLabel?: boolean;
  arrowMarkerAbsolute?: boolean;
  useMaxWidth?: boolean;
}

// C4 Diagram Configuration
export interface C4Config {
  diagramMarginX?: number;
  diagramMarginY?: number;
  c4ShapeMargin?: number;
  c4ShapePadding?: number;
  width?: number;
  height?: number;
  boxMargin?: number;
  useMaxWidth?: boolean;
  c4ShapeInRow?: number;
  c4BoundaryInRow?: number;
  personFontSize?: number;
  personFontFamily?: string;
  personFontWeight?: string;
  external_personFontSize?: number;
  external_personFontFamily?: string;
  external_personFontWeight?: string;
  systemFontSize?: number;
  systemFontFamily?: string;
  systemFontWeight?: string;
  external_systemFontSize?: number;
  external_systemFontFamily?: string;
  external_systemFontWeight?: string;
  system_dbFontSize?: number;
  system_dbFontFamily?: string;
  system_dbFontWeight?: string;
  external_system_dbFontSize?: number;
  external_system_dbFontFamily?: string;
  external_system_dbFontWeight?: string;
  system_queueFontSize?: number;
  system_queueFontFamily?: string;
  system_queueFontWeight?: string;
  external_system_queueFontSize?: number;
  external_system_queueFontFamily?: string;
  external_system_queueFontWeight?: string;
  boundaryFontSize?: number;
  boundaryFontFamily?: string;
  boundaryFontWeight?: string;
  messageFontSize?: number;
  messageFontFamily?: string;
  messageFontWeight?: string;
  containerFontSize?: number;
  containerFontFamily?: string;
  containerFontWeight?: string;
  external_containerFontSize?: number;
  external_containerFontFamily?: string;
  external_containerFontWeight?: string;
  container_dbFontSize?: number;
  container_dbFontFamily?: string;
  container_dbFontWeight?: string;
  external_container_dbFontSize?: number;
  external_container_dbFontFamily?: string;
  external_container_dbFontWeight?: string;
  container_queueFontSize?: number;
  container_queueFontFamily?: string;
  container_queueFontWeight?: string;
  external_container_queueFontSize?: number;
  external_container_queueFontFamily?: string;
  external_container_queueFontWeight?: string;
  componentFontSize?: number;
  componentFontFamily?: string;
  componentFontWeight?: string;
  external_componentFontSize?: number;
  external_componentFontFamily?: string;
  external_componentFontWeight?: string;
  component_dbFontSize?: number;
  component_dbFontFamily?: string;
  component_dbFontWeight?: string;
  external_component_dbFontSize?: number;
  external_component_dbFontFamily?: string;
  external_component_dbFontWeight?: string;
  component_queueFontSize?: number;
  component_queueFontFamily?: string;
  component_queueFontWeight?: string;
  external_component_queueFontSize?: number;
  external_component_queueFontFamily?: string;
  external_component_queueFontWeight?: string;
  wrap?: boolean;
  wrapPadding?: number;
  personFont?: Function;
  external_personFont?: Function;
  systemFont?: Function;
  external_systemFont?: Function;
  system_dbFont?: Function;
  external_system_dbFont?: Function;
  system_queueFont?: Function;
  external_system_queueFont?: Function;
  containerFont?: Function;
  external_containerFont?: Function;
  container_dbFont?: Function;
  external_container_dbFont?: Function;
  container_queueFont?: Function;
  external_container_queueFont?: Function;
  componentFont?: Function;
  external_componentFont?: Function;
  component_dbFont?: Function;
  external_component_dbFont?: Function;
  component_queueFont?: Function;
  external_component_queueFont?: Function;
  boundaryFont?: Function;
  messageFont?: Function;
  person_bg_color?: string;
  person_border_color?: string;
  external_person_bg_color?: string;
  external_person_border_color?: string;
  system_bg_color?: string;
  system_border_color?: string;
  system_db_bg_color?: string;
  system_db_border_color?: string;
  system_queue_bg_color?: string;
  system_queue_border_color?: string;
  external_system_bg_color?: string;
  external_system_border_color?: string;
  external_system_db_bg_color?: string;
  external_system_db_border_color?: string;
  external_system_queue_bg_color?: string;
  external_system_queue_border_color?: string;
  container_bg_color?: string;
  container_border_color?: string;
  container_db_bg_color?: string;
  container_db_border_color?: string;
  container_queue_bg_color?: string;
  container_queue_border_color?: string;
  external_container_bg_color?: string;
  external_container_border_color?: string;
  external_container_db_bg_color?: string;
  external_container_db_border_color?: string;
  external_container_queue_bg_color?: string;
  external_container_queue_border_color?: string;
  component_bg_color?: string;
  component_border_color?: string;
  component_db_bg_color?: string;
  component_db_border_color?: string;
  component_queue_bg_color?: string;
  component_queue_border_color?: string;
  external_component_bg_color?: string;
  external_component_border_color?: string;
  external_component_db_bg_color?: string;
  external_component_db_border_color?: string;
  external_component_queue_bg_color?: string;
  external_component_queue_border_color?: string;
}

// Sankey Diagram Configuration
export interface SankeyConfig {
  width?: number;
  height?: number;
  linkColor?: string;
  nodeAlignment?: string;
  useMaxWidth?: boolean;
  showValues?: boolean;
  prefix?: string;
  suffix?: string;
}

// Packet Diagram Configuration
export interface PacketConfig {
  showBits?: boolean;
  bitsPerRow?: number;
  useMaxWidth?: boolean;
}

// Block Diagram Configuration
export interface BlockConfig {
  padding?: number;
  useMaxWidth?: boolean;
}

// Radar Chart Configuration
export interface RadarConfig {
  useMaxWidth?: boolean;
}

// Main Mermaid Configuration
export interface MermaidConfig {
  // Global settings
  theme?: string;
  themeVariables?: Record<string, unknown>;
  themeCSS?: string;
  look?: Look;
  handDrawnSeed?: number;
  layout?: string;
  maxTextSize?: number;
  maxEdges?: number;
  elk?: Record<string, unknown>;
  darkMode?: boolean;
  htmlLabels?: boolean;
  fontFamily?: string;
  altFontFamily?: string;
  logLevel?: LogLevel;
  securityLevel?: SecurityLevel;
  startOnLoad?: boolean;
  arrowMarkerAbsolute?: boolean;
  secure?: string[];
  legacyMathML?: boolean;
  forceLegacyMathML?: boolean;
  deterministicIds?: boolean;
  deterministicIDSeed?: string;
  dompurifyConfig?: Record<string, unknown>;
  wrap?: boolean;
  fontSize?: number;
  markdownAutoWrap?: boolean;
  suppressErrorRendering?: boolean;

  // Per-diagram-type configurations
  flowchart?: FlowchartConfig;
  sequence?: SequenceConfig;
  gantt?: GanttConfig;
  journey?: JourneyConfig;
  timeline?: TimelineConfig;
  class?: ClassConfig;
  state?: StateConfig;
  er?: ERConfig;
  pie?: PieConfig;
  quadrantChart?: QuadrantChartConfig;
  xyChart?: XYChartConfig;
  requirement?: RequirementConfig;
  architecture?: ArchitectureConfig;
  mindmap?: MindmapConfig;
  kanban?: KanbanConfig;
  gitGraph?: GitGraphConfig;
  c4?: C4Config;
  sankey?: SankeyConfig;
  packet?: PacketConfig;
  block?: BlockConfig;
  radar?: RadarConfig;
}
