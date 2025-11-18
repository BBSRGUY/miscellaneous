'use client';

import DiagramWorkspace from '@/components/DiagramWorkspace';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState } from 'react';

export default function Home() {
  const [queryClient] = useState(() => new QueryClient());

  return (
    <QueryClientProvider client={queryClient}>
      <main className="h-screen flex flex-col">
        <DiagramWorkspace />
      </main>
    </QueryClientProvider>
  );
}
