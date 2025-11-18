'use client';

import { useDiagramStore } from '@/store/diagramStore';
import { useEffect, useRef, useState } from 'react';
import mermaid from 'mermaid';
import { Download, ZoomIn, ZoomOut, RefreshCw } from 'lucide-react';

export default function DiagramPreview() {
  const { getCurrentDiagram, mermaidConfig } = useDiagramStore();
  const currentDiagram = getCurrentDiagram();
  const [svg, setSvg] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [zoom, setZoom] = useState(100);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    mermaid.initialize({
      ...mermaidConfig,
      startOnLoad: false,
      securityLevel: 'loose',
    });
  }, [mermaidConfig]);

  useEffect(() => {
    const renderDiagram = async () => {
      if (!currentDiagram?.mermaidCode) {
        setSvg('');
        setError(null);
        return;
      }

      try {
        setError(null);
        const { svg: renderedSvg } = await mermaid.render(
          `diagram-${currentDiagram.id}`,
          currentDiagram.mermaidCode,
        );
        setSvg(renderedSvg);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to render diagram');
        setSvg('');
      }
    };

    const timer = setTimeout(renderDiagram, 300);
    return () => clearTimeout(timer);
  }, [currentDiagram?.mermaidCode, currentDiagram?.id]);

  const handleExport = async (format: 'png' | 'svg' | 'jpg') => {
    if (!svg) return;

    if (format === 'svg') {
      const blob = new Blob([svg], { type: 'image/svg+xml' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${currentDiagram?.title || 'diagram'}.svg`;
      a.click();
      URL.revokeObjectURL(url);
    } else {
      // Convert SVG to PNG/JPG using canvas
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      const img = new Image();

      img.onload = () => {
        canvas.width = img.width;
        canvas.height = img.height;

        if (ctx) {
          if (format === 'jpg') {
            ctx.fillStyle = 'white';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
          }
          ctx.drawImage(img, 0, 0);

          canvas.toBlob((blob) => {
            if (blob) {
              const url = URL.createObjectURL(blob);
              const a = document.createElement('a');
              a.href = url;
              a.download = `${currentDiagram?.title || 'diagram'}.${format}`;
              a.click();
              URL.revokeObjectURL(url);
            }
          }, `image/${format}`);
        }
      };

      const svgBlob = new Blob([svg], { type: 'image/svg+xml' });
      img.src = URL.createObjectURL(svgBlob);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="h-10 border-b border-border flex items-center justify-between px-4 bg-muted">
        <h2 className="text-sm font-semibold">Diagram Preview</h2>

        <div className="flex items-center gap-1">
          <button
            onClick={() => setZoom(Math.max(25, zoom - 25))}
            className="p-1.5 rounded hover:bg-background"
            title="Zoom out"
          >
            <ZoomOut size={16} />
          </button>

          <span className="text-xs px-2">{zoom}%</span>

          <button
            onClick={() => setZoom(Math.min(200, zoom + 25))}
            className="p-1.5 rounded hover:bg-background"
            title="Zoom in"
          >
            <ZoomIn size={16} />
          </button>

          <button
            onClick={() => setZoom(100)}
            className="p-1.5 rounded hover:bg-background"
            title="Reset zoom"
          >
            <RefreshCw size={16} />
          </button>

          <div className="h-4 w-px bg-border mx-1" />

          <div className="relative group">
            <button
              className="p-1.5 rounded hover:bg-background flex items-center gap-1"
              title="Export"
            >
              <Download size={16} />
            </button>

            <div className="absolute right-0 mt-1 bg-background border border-border rounded shadow-lg hidden group-hover:block z-10">
              <button
                onClick={() => handleExport('svg')}
                className="block w-full px-3 py-1.5 text-sm text-left hover:bg-secondary"
              >
                Export as SVG
              </button>
              <button
                onClick={() => handleExport('png')}
                className="block w-full px-3 py-1.5 text-sm text-left hover:bg-secondary"
              >
                Export as PNG
              </button>
              <button
                onClick={() => handleExport('jpg')}
                className="block w-full px-3 py-1.5 text-sm text-left hover:bg-secondary"
              >
                Export as JPG
              </button>
            </div>
          </div>
        </div>
      </div>

      <div
        ref={containerRef}
        className="flex-1 overflow-auto bg-background p-4 flex items-center justify-center"
      >
        {error ? (
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-4 max-w-2xl">
            <h3 className="text-red-800 dark:text-red-200 font-semibold mb-2">
              Render Error
            </h3>
            <pre className="text-sm text-red-600 dark:text-red-300 whitespace-pre-wrap">
              {error}
            </pre>
          </div>
        ) : svg ? (
          <div
            style={{ transform: `scale(${zoom / 100})`, transformOrigin: 'center' }}
            dangerouslySetInnerHTML={{ __html: svg }}
          />
        ) : (
          <div className="text-muted-foreground text-sm">
            Write Mermaid code in the editor to see the preview
          </div>
        )}
      </div>
    </div>
  );
}
