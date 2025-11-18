# Diagramlab - AI-Powered Mermaid IDE

A comprehensive, modern IDE for creating and editing Mermaid diagrams with AI assistance. Built with Next.js, NestJS, and designed for professional diagram creation.

## Features

### 🎨 Three-Pane Interface
- **Code Editor**: Monaco-powered editor with syntax support for Mermaid
- **Live Preview**: Real-time diagram rendering with zoom/pan controls
- **AI Assistant**: Chat interface for diagram generation and modification

### 📊 Comprehensive Diagram Support
Supports all major Mermaid diagram types:
- Flowchart, Sequence, Class, State, ER diagrams
- Gantt, User Journey, Timeline diagrams
- Pie, Quadrant, XY Chart, Radar charts
- C4, GitGraph, Mindmap, Kanban boards
- Architecture, Block, Packet, Sankey, Treemap diagrams

### 🤖 AI Integration (Coming Soon)
- Generate diagrams from natural language descriptions
- Modify existing diagrams with AI suggestions
- Explain diagram structure and components
- Convert between diagram types

### 💾 Export Capabilities
- Export to PNG, JPG, SVG formats
- Client-side rendering with quality controls
- Server-side export (planned)

### ⚙️ Advanced Configuration
- Full Mermaid configuration support
- Per-diagram-type configuration options
- Theme customization (light/dark mode)
- Layout and styling controls

## Tech Stack

### Backend
- **Framework**: NestJS (Node.js 20+)
- **Database**: PostgreSQL with Prisma ORM
- **WebSocket**: Socket.IO for real-time AI streaming
- **API**: RESTful endpoints with TypeScript

### Frontend
- **Framework**: Next.js 15 + React 19
- **Editor**: Monaco Editor
- **State Management**: Zustand + React Query
- **Styling**: Tailwind CSS
- **Diagram Rendering**: Mermaid.js

### Monorepo
- **Package Manager**: pnpm with workspaces
- **Build System**: Turbo
- **Shared Packages**: TypeScript packages for types, config, and AI utilities

## Project Structure

```
diagramlab/
├── apps/
│   ├── api/              # NestJS backend
│   │   ├── src/
│   │   │   ├── modules/
│   │   │   │   ├── project/
│   │   │   │   ├── diagram/
│   │   │   │   ├── ai-chat/
│   │   │   │   ├── export/
│   │   │   │   ├── mermaid-config/
│   │   │   │   └── health/
│   │   │   ├── prisma/
│   │   │   ├── app.module.ts
│   │   │   └── main.ts
│   │   └── package.json
│   │
│   └── web/              # Next.js frontend
│       ├── src/
│       │   ├── app/
│       │   ├── components/
│       │   │   ├── DiagramWorkspace.tsx
│       │   │   ├── CodeEditor.tsx
│       │   │   ├── DiagramPreview.tsx
│       │   │   ├── AIAssistant.tsx
│       │   │   └── Header.tsx
│       │   └── store/
│       │       ├── diagramStore.ts
│       │       └── chatStore.ts
│       └── package.json
│
├── packages/
│   ├── core/             # Shared types and interfaces
│   ├── mermaid-config/   # Mermaid configuration types & defaults
│   └── ai/               # AI prompt builders and LLM abstractions
│
├── prisma/
│   ├── schema.prisma     # Database schema
│   └── .env.example
│
├── package.json
├── pnpm-workspace.yaml
├── turbo.json
└── tsconfig.base.json
```

## Getting Started

### Prerequisites
- Node.js 20+
- pnpm 9+
- PostgreSQL 14+ (or use SQLite for local development)

### Installation

1. **Clone the repository**
```bash
git clone <repository-url>
cd diagramlab
```

2. **Install dependencies**
```bash
pnpm install
```

3. **Set up environment variables**

For the API:
```bash
cd apps/api
cp .env.example .env
```

Edit `apps/api/.env`:
```env
PORT=3001
NODE_ENV=development
DATABASE_URL="postgresql://user:password@localhost:5432/diagramlab?schema=public"
CORS_ORIGIN=http://localhost:3000
```

For Prisma:
```bash
cd prisma
cp .env.example .env
```

Edit `prisma/.env` with your database connection string.

4. **Set up the database**
```bash
# Generate Prisma client
pnpm db:generate

# Run migrations
pnpm db:migrate
```

5. **Build shared packages**
```bash
pnpm build
```

### Development

Start all services in development mode:
```bash
pnpm dev
```

This will start:
- Frontend at `http://localhost:3000`
- Backend API at `http://localhost:3001`

Or run individually:
```bash
# Frontend only
cd apps/web
pnpm dev

# Backend only
cd apps/api
pnpm dev
```

### Available Scripts

**Root-level commands:**
- `pnpm dev` - Start all apps in development mode
- `pnpm build` - Build all apps and packages
- `pnpm test` - Run tests across all packages
- `pnpm lint` - Lint all code
- `pnpm clean` - Clean all build artifacts
- `pnpm db:migrate` - Run database migrations
- `pnpm db:generate` - Generate Prisma client
- `pnpm db:studio` - Open Prisma Studio

## Database Schema

### Entities

**User**
- Authentication and ownership

**Project**
- Container for related diagrams
- Belongs to a user

**Diagram**
- Stores Mermaid code and configuration
- Linked to a project
- Supports all diagram types

**DiagramRevision**
- Version history for diagrams
- Created automatically on code changes

**ChatSession**
- AI chat sessions
- Linked to projects and optionally diagrams

**ChatMessage**
- Individual messages in chat sessions
- Stores AI responses and generated Mermaid code

## API Endpoints

### Projects
- `GET /projects` - List all projects
- `POST /projects` - Create project
- `GET /projects/:id` - Get project with diagrams
- `PATCH /projects/:id` - Update project
- `DELETE /projects/:id` - Delete project

### Diagrams
- `GET /diagrams?projectId=:id` - List diagrams by project
- `POST /diagrams` - Create diagram
- `GET /diagrams/:id` - Get diagram with revisions
- `PATCH /diagrams/:id` - Update diagram
- `DELETE /diagrams/:id` - Delete diagram

### AI Chat
- `POST /chat/sessions` - Create chat session
- `GET /chat/sessions/:id` - Get session
- `GET /chat/sessions/:id/messages` - Get messages
- `POST /chat/messages` - Create message
- WebSocket: `/chat/sessions/:id/stream` - Stream AI responses

### Export
- `POST /export/diagram/:id?format=png|svg|jpg` - Export diagram

### Mermaid Config
- `GET /mermaid-config/default` - Get default config
- `GET /mermaid-config/default-for-type?type=flowchart` - Get type-specific config

### Health
- `GET /health` - Health check with database status
- `GET /health/ping` - Simple ping

## Roadmap

### Phase 1: Core Infrastructure ✅
- [x] Monorepo setup
- [x] Frontend three-pane layout
- [x] Backend API with CRUD operations
- [x] Mermaid rendering
- [x] Basic state management

### Phase 2: Configuration & Persistence (Next)
- [ ] Wire frontend to backend API
- [ ] Implement auto-save
- [ ] Config editor UI
- [ ] Theme persistence

### Phase 3: AI Integration (Planned)
- [ ] Implement LLM providers (OpenAI, Anthropic)
- [ ] WebSocket streaming
- [ ] Prompt engineering
- [ ] Code generation and modification

### Phase 4: Export & Advanced Features (Planned)
- [ ] Server-side export (PNG/SVG/PDF)
- [ ] Diagram templates library
- [ ] Collaborative editing
- [ ] Diagram sharing

### Phase 5: Production (Planned)
- [ ] Authentication (OAuth)
- [ ] User management
- [ ] Deployment configuration
- [ ] CI/CD pipeline
- [ ] Documentation site

## Configuration

### Mermaid Configuration
The app supports comprehensive Mermaid configuration including:

**Global Settings:**
- Theme (default, dark, forest, neutral)
- Look (classic, handDrawn)
- Font family and sizing
- Security level
- Layout options

**Per-Diagram-Type Settings:**
Each diagram type has specific configuration options. See `packages/mermaid-config/src/types.ts` for full details.

## Development Guidelines

### Code Style
- TypeScript strict mode enabled
- ESLint + Prettier for formatting
- Follow Airbnb style guide principles

### Testing
```bash
# Run all tests
pnpm test

# Frontend tests
cd apps/web
pnpm test

# Backend tests
cd apps/api
pnpm test
```

### Adding a New Diagram Type
1. Add type to `DiagramTypeSchema` in `packages/core/src/types.ts`
2. Add config interface in `packages/mermaid-config/src/types.ts`
3. Add defaults in `packages/mermaid-config/src/defaults.ts`
4. Update UI components to support new type

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

MIT License - see LICENSE file for details

---

Built with ❤️ using Next.js, NestJS, and Mermaid.js