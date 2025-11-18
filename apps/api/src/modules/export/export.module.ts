import { Module } from '@nestjs/common';
import { ExportController } from './export.controller';
import { ExportService } from './export.service';
import { DiagramModule } from '../diagram/diagram.module';

@Module({
  imports: [DiagramModule],
  controllers: [ExportController],
  providers: [ExportService],
})
export class ExportModule {}
