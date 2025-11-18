import { Controller, Post, Param, Query } from '@nestjs/common';
import { ExportService } from './export.service';
import type { ExportFormat } from '@diagramlab/core';

@Controller('export')
export class ExportController {
  constructor(private readonly exportService: ExportService) {}

  @Post('diagram/:id')
  async exportDiagram(@Param('id') id: string, @Query('format') format: ExportFormat) {
    return this.exportService.exportDiagram(id, format);
  }
}
