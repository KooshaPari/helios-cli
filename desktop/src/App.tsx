import { useState, useCallback } from 'react';
import Sidebar from './components/Sidebar';
import Header from './components/Header';
import Chat from './components/Chat';
import CommandPalette from './components/CommandPalette';
import StatusBar from './components/StatusBar';
import './App.css';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
}

export interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
}

function App() {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeConversationId, setActiveConversationId] = useState<string | null>(null);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected' | 'connecting'>('disconnected');

  const activeConversation = conversations.find(c => c.id === activeConversationId) || null;

  const createNewConversation = useCallback(() => {
    const newConversation: Conversation = {
      id: `conv-${Date.now()}`,
      title: `New Conversation`,
      messages: [],
      createdAt: new Date(),
    };
    setConversations(prev => [newConversation, ...prev]);
    setActiveConversationId(newConversation.id);
    return newConversation.id;
  }, []);

  const sendMessage = useCallback(async (content: string) => {
    if (!content.trim()) return;

    let conversationId = activeConversationId;
    if (!conversationId) {
      conversationId = createNewConversation();
    }

    const userMessage: Message = {
      id: `msg-${Date.now()}`,
      role: 'user',
      content,
      timestamp: new Date(),
    };

    setConversations(prev =>
      prev.map(conv =>
        conv.id === conversationId
          ? { ...conv, messages: [...conv.messages, userMessage] }
          : conv
      )
    );

    setIsProcessing(true);

    // Simulate AI response (in real app, this would call Tauri backend)
    setTimeout(() => {
      const assistantMessage: Message = {
        id: `msg-${Date.now() + 1}`,
        role: 'assistant',
        content: `I received your message: "${content}"\n\nThis is a placeholder response. In the full Helios CLI, I would:\n\n1. **Process your request** using the Helios AI engine\n2. **Generate a response** with proper formatting\n3. **Execute commands** if requested\n\nFor now, this demonstrates the UI layout and component structure.`,
        timestamp: new Date(),
      };

      setConversations(prev =>
        prev.map(conv =>
          conv.id === conversationId
            ? { ...conv, messages: [...conv.messages, assistantMessage] }
            : conv
        )
      );
      setIsProcessing(false);
    }, 1000);
  }, [activeConversationId, createNewConversation]);

  const handleCommand = useCallback((command: string) => {
    const parts = command.split(' ');
    const cmd = parts[0]?.toLowerCase();

    switch (cmd) {
      case '/new':
      case '/clear':
        createNewConversation();
        break;
      case '/help':
        // Show help in chat
        const helpMessage: Message = {
          id: `msg-${Date.now()}`,
          role: 'system',
          content: `**Available Commands:**\n\n- \`/new\` or \`/clear\` - Start a new conversation\n- \`/help\` - Show this help message\n- \`/status\` - Show connection status\n- \`/quit\` - Close the application`,
          timestamp: new Date(),
        };
        if (activeConversationId) {
          setConversations(prev =>
            prev.map(conv =>
              conv.id === activeConversationId
                ? { ...conv, messages: [...conv.messages, helpMessage] }
                : conv
            )
          );
        }
        break;
      case '/status':
        const statusMessage: Message = {
          id: `msg-${Date.now()}`,
          role: 'system',
          content: `**Connection Status:** ${connectionStatus}\n**Active Conversations:** ${conversations.length}\n**Current Conversation:** ${activeConversation?.title || 'None'}`,
          timestamp: new Date(),
        };
        if (activeConversationId) {
          setConversations(prev =>
            prev.map(conv =>
              conv.id === activeConversationId
                ? { ...conv, messages: [...conv.messages, statusMessage] }
                : conv
            )
          );
        }
        break;
      default:
        sendMessage(command);
    }
  }, [activeConversationId, activeConversation, conversations.length, connectionStatus, sendMessage, createNewConversation]);

  const selectConversation = useCallback((id: string) => {
    setActiveConversationId(id);
  }, []);

  const deleteConversation = useCallback((id: string) => {
    setConversations(prev => prev.filter(conv => conv.id !== id));
    if (activeConversationId === id) {
      setActiveConversationId(null);
    }
  }, [activeConversationId]);

  // Keyboard shortcut for command palette
  useState(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsCommandPaletteOpen(prev => !prev);
      }
      if (e.key === 'Escape') {
        setIsCommandPaletteOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  return (
    <div className="app">
      <Sidebar
        conversations={conversations}
        activeConversationId={activeConversationId}
        onSelectConversation={selectConversation}
        onNewConversation={createNewConversation}
        onDeleteConversation={deleteConversation}
      />
      <div className="main-content">
        <Header
          title={activeConversation?.title || 'Helios CLI Desktop'}
          isProcessing={isProcessing}
          onOpenCommandPalette={() => setIsCommandPaletteOpen(true)}
        />
        <Chat
          conversation={activeConversation}
          isProcessing={isProcessing}
          onSendMessage={sendMessage}
        />
        <StatusBar
          connectionStatus={connectionStatus}
          messageCount={activeConversation?.messages.length || 0}
        />
      </div>
      {isCommandPaletteOpen && (
        <CommandPalette
          onClose={() => setIsCommandPaletteOpen(false)}
          onCommand={handleCommand}
        />
      )}
    </div>
  );
}

export default App;