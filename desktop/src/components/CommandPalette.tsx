import { useState, useRef, useEffect } from 'react';
import './CommandPalette.css';

interface CommandPaletteProps {
  onClose: () => void;
  onCommand: (command: string) => void;
}

const commands = [
  { id: 'new', label: 'New Conversation', shortcut: 'Ctrl+N', icon: '📝' },
  { id: 'clear', label: 'Clear Conversation', shortcut: 'Ctrl+L', icon: '🗑️' },
  { id: 'help', label: 'Show Help', shortcut: 'Ctrl+H', icon: '❓' },
  { id: 'status', label: 'Connection Status', shortcut: 'Ctrl+I', icon: '📊' },
  { id: 'settings', label: 'Settings', shortcut: 'Ctrl+,', icon: '⚙️' },
  { id: 'about', label: 'About Helios CLI', shortcut: 'F1', icon: 'ℹ️' },
];

function CommandPalette({ onClose, onCommand }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const filteredCommands = commands.filter((cmd) =>
    cmd.label.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev < filteredCommands.length - 1 ? prev + 1 : 0
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev > 0 ? prev - 1 : filteredCommands.length - 1
        );
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          handleSelectCommand(filteredCommands[selectedIndex].id);
        }
        break;
      case 'Escape':
        onClose();
        break;
    }
  };

  const handleSelectCommand = (commandId: string) => {
    const command = commands.find((cmd) => cmd.id === commandId);
    if (command) {
      onCommand(`/${command.id}`);
    }
    onClose();
  };

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div
        className="command-palette"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="palette-header">
          <span className="palette-icon">⌘</span>
          <input
            ref={inputRef}
            className="palette-input"
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command..."
          />
        </div>
        <div className="palette-divider"></div>
        <div className="palette-results">
          {filteredCommands.length === 0 ? (
            <div className="no-results">No commands found</div>
          ) : (
            filteredCommands.map((command, index) => (
              <div
                key={command.id}
                className={`palette-item ${
                  index === selectedIndex ? 'selected' : ''
                }`}
                onClick={() => handleSelectCommand(command.id)}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <span className="item-icon">{command.icon}</span>
                <span className="item-label">{command.label}</span>
                <kbd className="item-shortcut">{command.shortcut}</kbd>
              </div>
            ))
          )}
        </div>
        <div className="palette-footer">
          <span className="footer-hint">
            <kbd>↑↓</kbd> Navigate
          </span>
          <span className="footer-hint">
            <kbd>↵</kbd> Select
          </span>
          <span className="footer-hint">
            <kbd>esc</kbd> Close
          </span>
        </div>
      </div>
    </div>
  );
}

export default CommandPalette;