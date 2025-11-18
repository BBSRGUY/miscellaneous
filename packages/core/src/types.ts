import { z } from 'zod';

// Diagram Types
export const DiagramTypeSchema = z.enum([
  'flowchart',
  'sequence',
  'class',
  'state',
  'er',
  'user-journey',
  'gantt',
  'pie',
  'quadrant',
  'requirement',
  'gitgraph',
  'c4',
  'mindmap',
  'timeline',
  'zenuml',
  'sankey',
  'xy-chart',
  'block',
  'packet',
  'kanban',
  'architecture',
  'radar',
  'treemap',
]);

export type DiagramType = z.infer<typeof DiagramTypeSchema>;

// User
export interface User {
  id: string;
  email: string;
  name: string;
  authProvider?: string;
  createdAt: Date;
  updatedAt: Date;
}

// Project
export interface Project {
  id: string;
  ownerId: string;
  name: string;
  description?: string;
  createdAt: Date;
  updatedAt: Date;
}

// Diagram
export interface Diagram {
  id: string;
  projectId: string;
  title: string;
  diagramType: DiagramType;
  mermaidCode: string;
  mermaidConfig: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
}

// Chat
export const ChatRoleSchema = z.enum(['system', 'user', 'assistant']);
export type ChatRole = z.infer<typeof ChatRoleSchema>;

export interface ChatSession {
  id: string;
  projectId: string;
  diagramId?: string;
  model: string;
  createdAt: Date;
}

export interface ChatMessage {
  id: string;
  sessionId: string;
  role: ChatRole;
  content: string;
  mermaidCode?: string;
  createdAt: Date;
}

// Diagram Revision
export interface DiagramRevision {
  id: string;
  diagramId: string;
  mermaidCode: string;
  mermaidConfig: Record<string, unknown>;
  createdAt: Date;
  authorId?: string;
}

// DTOs
export interface CreateProjectDto {
  name: string;
  description?: string;
}

export interface UpdateProjectDto {
  name?: string;
  description?: string;
}

export interface CreateDiagramDto {
  projectId: string;
  title: string;
  diagramType: DiagramType;
  mermaidCode?: string;
  mermaidConfig?: Record<string, unknown>;
}

export interface UpdateDiagramDto {
  title?: string;
  diagramType?: DiagramType;
  mermaidCode?: string;
  mermaidConfig?: Record<string, unknown>;
}

export interface CreateChatSessionDto {
  projectId: string;
  diagramId?: string;
  model: string;
}

export interface CreateChatMessageDto {
  sessionId: string;
  role: ChatRole;
  content: string;
  mermaidCode?: string;
}

// Export formats
export const ExportFormatSchema = z.enum(['png', 'jpg', 'svg', 'pdf']);
export type ExportFormat = z.infer<typeof ExportFormatSchema>;

export interface ExportDiagramDto {
  diagramId: string;
  format: ExportFormat;
}
