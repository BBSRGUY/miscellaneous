import { Module } from '@nestjs/common';
import { MermaidConfigController } from './mermaid-config.controller';

@Module({
  controllers: [MermaidConfigController],
})
export class MermaidConfigModule {}
