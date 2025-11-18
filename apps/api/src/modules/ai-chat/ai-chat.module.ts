import { Module } from '@nestjs/common';
import { AIChatController } from './ai-chat.controller';
import { AIChatService } from './ai-chat.service';
import { AIChatGateway } from './ai-chat.gateway';

@Module({
  controllers: [AIChatController],
  providers: [AIChatService, AIChatGateway],
})
export class AIChatModule {}
