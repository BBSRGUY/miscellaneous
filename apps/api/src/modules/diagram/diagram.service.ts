import { Injectable } from '@nestjs/common';
import { PrismaService } from '../../prisma/prisma.service';
import type { CreateDiagramDto, UpdateDiagramDto } from '@diagramlab/core';
import { getDefaultConfigForType } from '@diagramlab/mermaid-config';

@Injectable()
export class DiagramService {
  constructor(private prisma: PrismaService) {}

  async create(dto: CreateDiagramDto) {
    const defaultConfig = getDefaultConfigForType(dto.diagramType);

    return this.prisma.diagram.create({
      data: {
        projectId: dto.projectId,
        title: dto.title,
        diagramType: dto.diagramType,
        mermaidCode: dto.mermaidCode || '',
        mermaidConfig: (dto.mermaidConfig as any) || defaultConfig,
      },
    });
  }

  async findByProject(projectId: string) {
    return this.prisma.diagram.findMany({
      where: { projectId },
      orderBy: { updatedAt: 'desc' },
    });
  }

  async findOne(id: string) {
    return this.prisma.diagram.findUnique({
      where: { id },
      include: {
        revisions: {
          take: 10,
          orderBy: { createdAt: 'desc' },
        },
      },
    });
  }

  async update(id: string, dto: UpdateDiagramDto) {
    const diagram = await this.prisma.diagram.findUnique({ where: { id } });

    if (!diagram) {
      throw new Error('Diagram not found');
    }

    // Create revision if code changed
    if (dto.mermaidCode && dto.mermaidCode !== diagram.mermaidCode) {
      await this.prisma.diagramRevision.create({
        data: {
          diagramId: id,
          mermaidCode: diagram.mermaidCode,
          mermaidConfig: diagram.mermaidConfig,
        },
      });
    }

    return this.prisma.diagram.update({
      where: { id },
      data: dto,
    });
  }

  async remove(id: string) {
    return this.prisma.diagram.delete({
      where: { id },
    });
  }
}
