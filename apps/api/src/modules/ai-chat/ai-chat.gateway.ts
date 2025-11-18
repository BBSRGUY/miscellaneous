import {
  WebSocketGateway,
  SubscribeMessage,
  MessageBody,
  ConnectedSocket,
  WebSocketServer,
} from '@nestjs/websockets';
import { Server, Socket } from 'socket.io';
import { AIChatService } from './ai-chat.service';

@WebSocketGateway({
  cors: {
    origin: process.env.CORS_ORIGIN || 'http://localhost:3000',
    credentials: true,
  },
})
export class AIChatGateway {
  @WebSocketServer()
  server: Server;

  constructor(private readonly chatService: AIChatService) {}

  @SubscribeMessage('chat:stream')
  async handleChatStream(
    @MessageBody() data: { sessionId: string; message: string },
    @ConnectedSocket() client: Socket,
  ) {
    try {
      // Save user message
      await this.chatService.createMessage({
        sessionId: data.sessionId,
        role: 'user',
        content: data.message,
      });

      // Stream AI response
      let fullResponse = '';
      for await (const chunk of this.chatService.streamCompletion(data.sessionId, data.message)) {
        fullResponse += chunk;
        client.emit('chat:chunk', { chunk });
      }

      // Save assistant message
      await this.chatService.createMessage({
        sessionId: data.sessionId,
        role: 'assistant',
        content: fullResponse,
      });

      client.emit('chat:complete', { message: fullResponse });
    } catch (error) {
      client.emit('chat:error', { error: error.message });
    }
  }
}
