import { Controller, Get, Post, Body, Patch, Param, Delete } from '@nestjs/common';
import { ProjectService } from './project.service';
import type { CreateProjectDto, UpdateProjectDto } from '@diagramlab/core';

@Controller('projects')
export class ProjectController {
  constructor(private readonly projectService: ProjectService) {}

  @Post()
  create(@Body() createProjectDto: CreateProjectDto) {
    // TODO: Get user ID from auth
    const ownerId = 'anonymous-user';
    return this.projectService.create(ownerId, createProjectDto);
  }

  @Get()
  findAll() {
    // TODO: Get user ID from auth
    const ownerId = 'anonymous-user';
    return this.projectService.findAll(ownerId);
  }

  @Get(':id')
  findOne(@Param('id') id: string) {
    return this.projectService.findOne(id);
  }

  @Patch(':id')
  update(@Param('id') id: string, @Body() updateProjectDto: UpdateProjectDto) {
    return this.projectService.update(id, updateProjectDto);
  }

  @Delete(':id')
  remove(@Param('id') id: string) {
    return this.projectService.remove(id);
  }
}
