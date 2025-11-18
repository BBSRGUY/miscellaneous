import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Diagramlab - AI-Powered Mermaid IDE',
  description: 'Create, edit, and export Mermaid diagrams with AI assistance',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
