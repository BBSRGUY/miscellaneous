import { Controller, Get, Post, Body, Param } from '@nestjs/common';
import { AIChatService } from './ai-chat.service';
import type { CreateChatSessionDto, CreateChatMessageDto } from '@diagramlab/core';

@Controller('chat')
export class AIChatController {
  constructor(private readonly chatService: AIChatService) {}

  @Post('sessions')
  createSession(@Body() dto: CreateChatSessionDto) {
    return this.chatService.createSession(dto);
  }

  @Get('sessions/:id')
  getSession(@Param('id') id: string) {
    return this.chatService.getSession(id);
  }

  @Get('sessions/:id/messages')
  getMessages(@Param('id') id: string) {
    return this.chatService.getMessages(id);
  }

  @Post('messages')
  createMessage(@Body() dto: CreateChatMessageDto) {
    return this.chatService.createMessage(dto);
  }
}
