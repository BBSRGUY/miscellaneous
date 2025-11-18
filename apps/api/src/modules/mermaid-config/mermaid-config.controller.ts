import { Controller, Get, Query } from '@nestjs/common';
import { getDefaultConfig, getDefaultConfigForType } from '@diagramlab/mermaid-config';
import type { DiagramType } from '@diagramlab/core';

@Controller('mermaid-config')
export class MermaidConfigController {
  @Get('default')
  getDefault() {
    return getDefaultConfig();
  }

  @Get('default-for-type')
  getDefaultForType(@Query('type') type: DiagramType) {
    return getDefaultConfigForType(type);
  }
}
