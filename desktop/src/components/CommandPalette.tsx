import { useState, useRef, useEffect } from 'react';
import type { AppView } from '../types';
import './CommandPalette.css';

interface CommandPaletteProps {
  onClose: () => void;
  onCommand: (command: string) => void;
  onNavigate?: (view: AppView) => void;
  onOpenSearch?: () => void;
}

interface CommandItem {
  id: string;
  label: string;
  shortcut?: string;
  icon: string;
  category: string;
}

const commands: CommandItem[] = [
  // General commands
  { id: 'new', label: 'New Conversation', shortcut: 'Ctrl+N', icon: '\u270d', category: 'General' },
  { id: 'clear', label: 'Clear Conversation', shortcut: 'Ctrl+L', icon: '\u{1f5d1}', category: 'General' },
  { id: 'help', label: 'Show Help', shortcut: 'Ctrl+H', icon: '\u2753', category: 'General' },
  { id: 'status', label: 'Connection Status', shortcut: 'Ctrl+I', icon: '\u{1f4ca}', category: 'General' },
  { id: 'settings', label: 'Settings', shortcut: 'Ctrl+,', icon: '\u2699', category: 'General' },
  { id: 'about', label: 'About Helios CLI', shortcut: 'F1', icon: '\u2139', category: 'General' },

  // Navigation commands
  { id: 'nav-dashboard', label: 'Go to Dashboard', icon: '\u2302', category: 'Navigation' },
  { id: 'nav-agents', label: 'Go to Agents', icon: '\u25cf', category: 'Navigation' },
  { id: 'nav-tasks', label: 'Go to Task Queue', icon: '\u25a0', category: 'Navigation' },
  { id: 'nav-tracera', label: 'Go to Tracera Board', icon: 'T', category: 'Navigation' },
  { id: 'nav-agileplus', label: 'Go to AgilePlus Scorecard', icon: 'A+', category: 'Navigation' },
  { id: 'nav-notifications', label: 'Go to Notifications', icon: '\u{1f514}', category: 'Navigation' },

  // Tracera commands
  { id: 'tracera-board', label: 'View Tracera Board', icon: 'T', category: 'Tracera' },
  { id: 'tracera-create', label: 'Create Tracera Issue', icon: '\u2795', category: 'Tracera' },

  // AgilePlus commands
  { id: 'agileplus-scorecard', label: 'View Scorecard', icon: 'A+', category: 'AgilePlus' },
  { id: 'agileplus-validate', label: 'Validate Spec', icon: '\u2714', category: 'AgilePlus' },

  // Helios commands
  { id: 'helios-spawn-agent', label: 'Spawn Agent', icon: '\u25cf', category: 'Helios' },
  { id: 'helios-create-task', label: 'Create Task', icon: '\u2795', category: 'Helios' },
  { id: 'helios-search', label: 'Unified Search', shortcut: 'Ctrl+K', icon: '\u{1f50d}', category: 'Helios' },
];

function CommandPalette({ onClose, onCommand, onNavigate, onOpenSearch }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const filteredCommands = commands.filter(
    (cmd) =>
      cmd.label.toLowerCase().includes(query.toLowerCase()) ||
      cmd.category.toLowerCase().includes(query.toLowerCase()),
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
          prev < filteredCommands.length - 1 ? prev + 1 : 0,
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev > 0 ? prev - 1 : filteredCommands.length - 1,
        );
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          handleSelectCommand(filteredCommands[selectedIndex]);
        }
        break;
      case 'Escape':
        onClose();
        break;
    }
  };

  const handleSelectCommand = (command: CommandItem) => {
    switch (command.id) {
      case 'nav-dashboard':
        onNavigate?.('chat');
        break;
      case 'nav-agents':
        onNavigate?.('agents');
        break;
      case 'nav-tasks':
        onNavigate?.('tasks');
        break;
      case 'nav-tracera':
        onNavigate?.('tracera-board');
        break;
      case 'nav-agileplus':
        onNavigate?.('agileplus-board');
        break;
      case 'nav-notifications':
        onNavigate?.('notifications');
        break;
      case 'tracera-board':
        onNavigate?.('tracera-board');
        break;
      case 'agileplus-scorecard':
        onNavigate?.('agileplus-board');
        break;
      case 'helios-search':
        onOpenSearch?.();
        break;
      default:
        onCommand(`/${command.id}`);
        break;
    }
    onClose();
  };

  // Group commands by category for display.
  const grouped: Record<string, CommandItem[]> = {};
  for (const cmd of filteredCommands) {
    if (!grouped[cmd.category]) grouped[cmd.category] = [];
    grouped[cmd.category].push(cmd);
  }

  let flatIndex = -1;

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div
        className="command-palette"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="palette-header">
          <span className="palette-icon">\u2318</span>
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
            Object.entries(grouped).map(([category, cmds]) => (
              <div key={category} className="palette-group">
                <div className="group-label">{category}</div>
                {cmds.map((command) => {
                  flatIndex++;
                  const idx = flatIndex;
                  return (
                    <div
                      key={command.id}
                      className={`palette-item ${
                        idx === selectedIndex ? 'selected' : ''
                      }`}
                      onClick={() => handleSelectCommand(command)}
                      onMouseEnter={() => setSelectedIndex(idx)}
                    >
                      <span className="item-icon">{command.icon}</span>
                      <span className="item-label">{command.label}</span>
                      {command.shortcut && (
                        <kbd className="item-shortcut">{command.shortcut}</kbd>
                      )}
                    </div>
                  );
                })}
              </div>
            ))
          )}
        </div>
        <div className="palette-footer">
          <span className="footer-hint">
            <kbd>\u2191\u2193</kbd> Navigate
          </span>
          <span className="footer-hint">
            <kbd>\u21b5</kbd> Select
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
