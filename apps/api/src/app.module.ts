import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { PrismaModule } from './prisma/prisma.module';
import { ProjectModule } from './modules/project/project.module';
import { DiagramModule } from './modules/diagram/diagram.module';
import { MermaidConfigModule } from './modules/mermaid-config/mermaid-config.module';
import { AIChatModule } from './modules/ai-chat/ai-chat.module';
import { ExportModule } from './modules/export/export.module';
import { HealthModule } from './modules/health/health.module';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),
    PrismaModule,
    ProjectModule,
    DiagramModule,
    MermaidConfigModule,
    AIChatModule,
    ExportModule,
    HealthModule,
  ],
})
export class AppModule {}
