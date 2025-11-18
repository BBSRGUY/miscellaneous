import { Injectable } from '@nestjs/common';
import { PrismaService } from '../../prisma/prisma.service';
import type { CreateChatSessionDto, CreateChatMessageDto } from '@diagramlab/core';

@Injectable()
export class AIChatService {
  constructor(private prisma: PrismaService) {}

  async createSession(dto: CreateChatSessionDto) {
    return this.prisma.chatSession.create({
      data: {
        projectId: dto.projectId,
        diagramId: dto.diagramId,
        model: dto.model,
      },
    });
  }

  async getSession(id: string) {
    return this.prisma.chatSession.findUnique({
      where: { id },
      include: {
        messages: {
          orderBy: { createdAt: 'asc' },
        },
      },
    });
  }

  async createMessage(dto: CreateChatMessageDto) {
    return this.prisma.chatMessage.create({
      data: {
        sessionId: dto.sessionId,
        role: dto.role,
        content: dto.content,
        mermaidCode: dto.mermaidCode,
      },
    });
  }

  async getMessages(sessionId: string) {
    return this.prisma.chatMessage.findMany({
      where: { sessionId },
      orderBy: { createdAt: 'asc' },
    });
  }

  // Placeholder for AI integration
  async *streamCompletion(sessionId: string, userMessage: string): AsyncIterable<string> {
    // TODO: Implement actual LLM provider integration
    // For now, just yield a simple response
    const response = `AI response to: "${userMessage}". Integration coming soon!`;

    for (const char of response) {
      yield char;
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
  }
}
