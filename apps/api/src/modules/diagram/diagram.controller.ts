import { Controller, Get, Post, Body, Patch, Param, Delete, Query } from '@nestjs/common';
import { DiagramService } from './diagram.service';
import type { CreateDiagramDto, UpdateDiagramDto } from '@diagramlab/core';

@Controller('diagrams')
export class DiagramController {
  constructor(private readonly diagramService: DiagramService) {}

  @Post()
  create(@Body() createDiagramDto: CreateDiagramDto) {
    return this.diagramService.create(createDiagramDto);
  }

  @Get()
  findByProject(@Query('projectId') projectId: string) {
    return this.diagramService.findByProject(projectId);
  }

  @Get(':id')
  findOne(@Param('id') id: string) {
    return this.diagramService.findOne(id);
  }

  @Patch(':id')
  update(@Param('id') id: string, @Body() updateDiagramDto: UpdateDiagramDto) {
    return this.diagramService.update(id, updateDiagramDto);
  }

  @Delete(':id')
  remove(@Param('id') id: string) {
    return this.diagramService.remove(id);
  }
}
