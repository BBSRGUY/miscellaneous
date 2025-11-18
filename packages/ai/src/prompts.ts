import type { DiagramType } from '@diagramlab/core';
import type { DiagramContext } from './types';

const SYSTEM_PROMPT = `You are Diagramlab AI, an expert assistant specialized in designing and editing diagrams using Mermaid syntax.

You support the following diagram types:
- flowchart: Flowchart diagrams with nodes and edges
- sequence: Sequence diagrams for interactions over time
- class: Class diagrams for object-oriented design
- state: State diagrams for state machines
- er: Entity-relationship diagrams for database design
- user-journey: User journey maps
- gantt: Gantt charts for project timelines
- pie: Pie charts for data visualization
- quadrant: Quadrant charts for 2x2 matrices
- requirement: Requirement diagrams
- gitgraph: Git branch and commit visualization
- c4: C4 architecture diagrams (Context, Container, Component, Code)
- mindmap: Mind maps for brainstorming
- timeline: Timeline diagrams for chronological events
- zenuml: ZenUML sequence diagrams (alternative syntax)
- sankey: Sankey diagrams for flow visualization
- xy-chart: XY scatter/line charts
- block: Block diagrams
- packet: Packet/protocol diagrams
- kanban: Kanban boards
- architecture: Architecture diagrams
- radar: Radar/spider charts
- treemap: Treemap visualizations

When asked to create or modify a diagram, you MUST:
1. Return valid Mermaid code for the requested type
2. Respond with a clear explanation followed by the Mermaid code
3. Format your response as JSON with this structure:
   {
     "responseText": "<explanation of what you did>",
     "mermaidCode": "<valid Mermaid diagram code>"
   }
4. Ensure the Mermaid code is properly formatted and syntactically correct
5. Do not include fence markers (\`\`\`mermaid) in the mermaidCode field
6. Keep diagrams simple, readable, and well-organized
7. Use meaningful node IDs and labels
8. Follow Mermaid best practices for the specific diagram type`;

export const buildSystemPrompt = (): string => {
  return SYSTEM_PROMPT;
};

export const buildGenerateDiagramPrompt = (
  diagramType: DiagramType,
  description: string,
  notes?: string,
): string => {
  let prompt = `Generate a ${diagramType} diagram based on the following description:\n\n${description}`;

  if (notes) {
    prompt += `\n\nAdditional notes: ${notes}`;
  }

  prompt += `\n\nProvide a clear, well-structured ${diagramType} diagram in Mermaid syntax.`;

  return prompt;
};

export const buildModifyDiagramPrompt = (
  diagramType: DiagramType,
  currentCode: string,
  instruction: string,
  config?: Record<string, unknown>,
): string => {
  let prompt = `Modify the following ${diagramType} diagram:\n\n${currentCode}\n\n`;
  prompt += `Instruction: ${instruction}\n\n`;

  if (config) {
    prompt += `Current configuration: ${JSON.stringify(config, null, 2)}\n\n`;
  }

  prompt += `Return the modified diagram in Mermaid syntax.`;

  return prompt;
};

export const buildExplainDiagramPrompt = (
  diagramType: DiagramType,
  currentCode: string,
  config?: Record<string, unknown>,
): string => {
  let prompt = `Explain the following ${diagramType} diagram in detail:\n\n${currentCode}\n\n`;

  if (config) {
    prompt += `Configuration: ${JSON.stringify(config, null, 2)}\n\n`;
  }

  prompt += `Provide a comprehensive explanation of:
1. The purpose and structure of this diagram
2. What each component represents
3. The relationships and flows shown
4. Any notable patterns or design decisions
5. Suggestions for improvements if applicable`;

  return prompt;
};

export const buildConvertDiagramPrompt = (
  sourceType: DiagramType,
  targetType: DiagramType,
  currentCode: string,
  instruction?: string,
): string => {
  let prompt = `Convert the following ${sourceType} diagram to a ${targetType} diagram:\n\n${currentCode}\n\n`;

  if (instruction) {
    prompt += `Additional instructions: ${instruction}\n\n`;
  }

  prompt += `Preserve the essential information and relationships while adapting to the ${targetType} diagram format.`;

  return prompt;
};

export const buildPromptFromContext = (context: DiagramContext): string => {
  switch (context.mode) {
    case 'generate':
      return buildGenerateDiagramPrompt(
        context.diagramType as DiagramType,
        context.instruction,
        context.notes,
      );

    case 'modify':
      if (!context.currentCode) {
        throw new Error('Current code is required for modify mode');
      }
      return buildModifyDiagramPrompt(
        context.diagramType as DiagramType,
        context.currentCode,
        context.instruction,
        context.config,
      );

    case 'explain':
      if (!context.currentCode) {
        throw new Error('Current code is required for explain mode');
      }
      return buildExplainDiagramPrompt(
        context.diagramType as DiagramType,
        context.currentCode,
        context.config,
      );

    case 'convert':
      if (!context.currentCode || !context.targetType) {
        throw new Error('Current code and target type are required for convert mode');
      }
      return buildConvertDiagramPrompt(
        context.diagramType as DiagramType,
        context.targetType as DiagramType,
        context.currentCode,
        context.instruction,
      );

    default:
      throw new Error(`Unknown mode: ${context.mode}`);
  }
};

export const cleanMermaidCode = (code: string): string => {
  // Remove fence markers
  let cleaned = code.replace(/^```mermaid\s*/gm, '').replace(/^```\s*/gm, '');

  // Trim whitespace
  cleaned = cleaned.trim();

  return cleaned;
};

export const parseLLMResponse = (response: string): { text: string; code?: string } => {
  try {
    // Try to parse as JSON first
    const parsed = JSON.parse(response);
    if (parsed.responseText && parsed.mermaidCode) {
      return {
        text: parsed.responseText,
        code: cleanMermaidCode(parsed.mermaidCode),
      };
    }
  } catch {
    // Not JSON, try to extract code from markdown fences
  }

  // Extract code from markdown fences
  const codeMatch = response.match(/```mermaid\s*([\s\S]*?)```/);
  if (codeMatch) {
    const code = cleanMermaidCode(codeMatch[1]);
    const text = response.replace(/```mermaid\s*[\s\S]*?```/, '').trim();
    return { text, code };
  }

  // No code found
  return { text: response };
};
