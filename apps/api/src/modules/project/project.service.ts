import { Injectable } from '@nestjs/common';
import { PrismaService } from '../../prisma/prisma.service';
import type { CreateProjectDto, UpdateProjectDto } from '@diagramlab/core';

@Injectable()
export class ProjectService {
  constructor(private prisma: PrismaService) {}

  async create(ownerId: string, dto: CreateProjectDto) {
    return this.prisma.project.create({
      data: {
        ownerId,
        name: dto.name,
        description: dto.description,
      },
    });
  }

  async findAll(ownerId: string) {
    return this.prisma.project.findMany({
      where: { ownerId },
      include: {
        _count: {
          select: { diagrams: true },
        },
      },
      orderBy: { updatedAt: 'desc' },
    });
  }

  async findOne(id: string) {
    return this.prisma.project.findUnique({
      where: { id },
      include: {
        diagrams: {
          orderBy: { updatedAt: 'desc' },
        },
      },
    });
  }

  async update(id: string, dto: UpdateProjectDto) {
    return this.prisma.project.update({
      where: { id },
      data: dto,
    });
  }

  async remove(id: string) {
    return this.prisma.project.delete({
      where: { id },
    });
  }
}
