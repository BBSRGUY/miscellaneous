import { useEffect, useRef } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import '@xterm/xterm/css/xterm.css';

const Terminal = () => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);

  useEffect(() => {
    if (!terminalRef.current) return;

    // Create xterm instance
    const xterm = new XTerm({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#d4d4d4',
        selection: 'rgba(255, 255, 255, 0.3)',
      },
      rows: 30,
      cols: 80,
    });

    // Create fit addon
    const fitAddon = new FitAddon();
    xterm.loadAddon(fitAddon);

    // Open terminal in DOM
    xterm.open(terminalRef.current);
    fitAddon.fit();

    // Store references
    xtermRef.current = xterm;
    fitAddonRef.current = fitAddon;

    // Welcome message
    xterm.writeln('Welcome to Forge CLI Terminal');
    xterm.writeln('');
    xterm.writeln('This is a placeholder terminal. In a full implementation,');
    xterm.writeln('this would connect to a PTY running the forge CLI.');
    xterm.writeln('');
    xterm.writeln('Try running commands in a real terminal:');
    xterm.writeln('  $ forge models list');
    xterm.writeln('  $ forge jobs list');
    xterm.writeln('  $ forge chat --model <model-id> --stream');
    xterm.writeln('');
    xterm.write('$ ');

    // Handle terminal input
    let currentLine = '';
    xterm.onData((data) => {
      const code = data.charCodeAt(0);

      if (code === 13) { // Enter
        xterm.writeln('');
        if (currentLine.trim()) {
          handleCommand(currentLine);
        }
        currentLine = '';
        xterm.write('$ ');
      } else if (code === 127) { // Backspace
        if (currentLine.length > 0) {
          currentLine = currentLine.slice(0, -1);
          xterm.write('\b \b');
        }
      } else if (code >= 32) { // Printable character
        currentLine += data;
        xterm.write(data);
      }
    });

    // Handle window resize
    const handleResize = () => {
      if (fitAddonRef.current && xtermRef.current) {
        fitAddonRef.current.fit();
        const dims = fitAddonRef.current.proposeDimensions();
        if (dims) {
          invoke('terminal_resize', { rows: dims.rows, cols: dims.cols }).catch(console.error);
        }
      }
    };

    window.addEventListener('resize', handleResize);

    // Spawn terminal process (placeholder)
    invoke('spawn_terminal').catch((err) => {
      console.error('Failed to spawn terminal:', err);
      xterm.writeln('\r\nError: Could not spawn terminal process');
    });

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize);
      xterm.dispose();
    };
  }, []);

  const handleCommand = (command: string) => {
    const xterm = xtermRef.current;
    if (!xterm) return;

    // Simple command handling for demo
    if (command === 'help') {
      xterm.writeln('Available commands:');
      xterm.writeln('  help    - Show this help message');
      xterm.writeln('  clear   - Clear the terminal');
      xterm.writeln('  status  - Check API status');
    } else if (command === 'clear') {
      xterm.clear();
    } else if (command === 'status') {
      xterm.writeln('Checking API status...');
      invoke('check_api_health')
        .then((health: any) => {
          xterm.writeln(`API Status: ${health.status}`);
          xterm.writeln(`API Version: ${health.version}`);
        })
        .catch((err) => {
          xterm.writeln(`Error: ${err}`);
        });
    } else {
      xterm.writeln(`Command not found: ${command}`);
      xterm.writeln('Type "help" for available commands');
    }

    // In a real implementation, send command to PTY
    // invoke('terminal_input', { input: command + '\n' }).catch(console.error);
  };

  return (
    <div className="terminal-container">
      <div ref={terminalRef} className="terminal" />
    </div>
  );
};

export default Terminal;
