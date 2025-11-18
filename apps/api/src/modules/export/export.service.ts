import { Injectable } from '@nestjs/common';
import { DiagramService } from '../diagram/diagram.service';
import type { ExportFormat } from '@diagramlab/core';

@Injectable()
export class ExportService {
  constructor(private diagramService: DiagramService) {}

  async exportDiagram(diagramId: string, format: ExportFormat) {
    const diagram = await this.diagramService.findOne(diagramId);

    if (!diagram) {
      throw new Error('Diagram not found');
    }

    // TODO: Implement server-side rendering using:
    // - Option A: @mermaid-js/mermaid-cli
    // - Option B: Playwright/Puppeteer for headless rendering

    // For now, return diagram data for client-side export
    return {
      diagram,
      format,
      message: 'Server-side export not yet implemented. Use client-side export for now.',
    };
  }
}
