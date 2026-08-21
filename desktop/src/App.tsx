import { useState, useCallback, useEffect } from 'react';
import Sidebar from './components/Sidebar';
import Header from './components/Header';
import Chat from './components/Chat';
import CommandPalette from './components/CommandPalette';
import StatusBar from './components/StatusBar';
import AgentPanel from './components/AgentPanel';
import TaskQueue from './components/TaskQueue';
import TraceraIntegration from './components/TraceraIntegration';
import AgilePlusIntegration from './components/AgilePlusIntegration';
import UnifiedSearch from './components/UnifiedSearch';
import NotificationCenter from './components/NotificationCenter';
import EmbeddedTracera from './components/EmbeddedTracera';
import EmbeddedAgilePlus from './components/EmbeddedAgilePlus';
import { useAgents, useTasks, useNotifications } from './hooks/useHelios';
import type { AppView } from './types';
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
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [isNotificationsOpen, setIsNotificationsOpen] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected' | 'connecting'>('disconnected');
  const [activeView, setActiveView] = useState<AppView>('chat');

  const { agents } = useAgents();
  const { tasks } = useTasks();
  const { counts } = useNotifications();

  useEffect(() => {
    // Simulate connection status.
    setConnectionStatus('connected');
  }, []);

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
        const helpMessage: Message = {
          id: `msg-${Date.now()}`,
          role: 'system',
          content: `**Available Commands:**\n\n- \`/new\` or \`/clear\` - Start a new conversation\n- \`/help\` - Show this help message\n- \`/status\` - Show connection status\n- \`/quit\` - Close the application\n\n**Milestone 3:**\n- Press **Ctrl+K** to open Unified Search\n- Use the bell icon to view Notifications\n- Navigate to Tracera Board and AgilePlus Scorecard from the sidebar`,
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
          content: `**Connection Status:** ${connectionStatus}\n**Active Conversations:** ${conversations.length}\n**Current Conversation:** ${activeConversation?.title || 'None'}\n**Unread Notifications:** ${counts?.unread || 0}`,
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
  }, [activeConversationId, activeConversation, conversations.length, connectionStatus, sendMessage, createNewConversation, counts]);

  const selectConversation = useCallback((id: string) => {
    setActiveConversationId(id);
  }, []);

  const deleteConversation = useCallback((id: string) => {
    setConversations(prev => prev.filter(conv => conv.id !== id));
    if (activeConversationId === id) {
      setActiveConversationId(null);
    }
  }, [activeConversationId]);

  // Keyboard shortcuts: Cmd/Ctrl+K for search, Cmd/Ctrl+Shift+K for command palette.
  useState(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        if (e.shiftKey) {
          setIsCommandPaletteOpen(prev => !prev);
        } else {
          setIsSearchOpen(prev => !prev);
        }
      }
      if ((e.metaKey || e.ctrlKey) && e.key === 'j') {
        e.preventDefault();
        setIsNotificationsOpen(prev => !prev);
      }
      if (e.key === 'Escape') {
        setIsCommandPaletteOpen(false);
        setIsSearchOpen(false);
        setIsNotificationsOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  // Determine header title based on active view.
  const getViewTitle = () => {
    switch (activeView) {
      case 'agents': return 'Agent Management';
      case 'tasks': return 'Task Queue';
      case 'tracera': return 'Tracera Integration';
      case 'agileplus': return 'AgilePlus Integration';
      case 'tracera-board': return 'Tracera Board';
      case 'agileplus-board': return 'AgilePlus Scorecard';
      case 'notifications': return 'Notifications';
      default: return activeConversation?.title || 'Helios CLI Desktop';
    }
  };

  // Render the main content area based on active view.
  const renderMainContent = () => {
    switch (activeView) {
      case 'agents':
        return <AgentPanel />;
      case 'tasks':
        return <TaskQueue />;
      case 'tracera':
        return <TraceraIntegration />;
      case 'agileplus':
        return <AgilePlusIntegration />;
      case 'tracera-board':
        return <EmbeddedTracera />;
      case 'agileplus-board':
        return <EmbeddedAgilePlus />;
      case 'notifications':
        return (
          <div className="notifications-view">
            <NotificationCenter isOpen={true} onClose={() => setActiveView('chat')} />
          </div>
        );
      default:
        return (
          <Chat
            conversation={activeConversation}
            isProcessing={isProcessing}
            onSendMessage={sendMessage}
          />
        );
    }
  };

  return (
    <div className="app">
      <Sidebar
        conversations={conversations}
        activeConversationId={activeConversationId}
        onSelectConversation={selectConversation}
        onNewConversation={createNewConversation}
        onDeleteConversation={deleteConversation}
        activeView={activeView}
        onNavigate={setActiveView}
        agentCount={agents.filter((a) => a.status === 'running').length}
        taskCount={tasks.filter((t) => t.status === 'pending').length}
        notificationCount={counts?.unread || 0}
      />
      <div className="main-content">
        <Header
          title={getViewTitle()}
          isProcessing={isProcessing}
          onOpenCommandPalette={() => setIsSearchOpen(true)}
          onOpenNotifications={() => setIsNotificationsOpen(prev => !prev)}
          notificationCount={counts?.unread || 0}
        />
        <div className="view-container">
          {renderMainContent()}
        </div>
        <StatusBar
          connectionStatus={connectionStatus}
          messageCount={activeConversation?.messages.length || 0}
          notificationCount={counts?.unread || 0}
        />
      </div>

      {/* Unified Search Overlay (Cmd+K) */}
      <UnifiedSearch
        isOpen={isSearchOpen}
        onClose={() => setIsSearchOpen(false)}
      />

      {/* Notification Center dropdown */}
      <NotificationCenter
        isOpen={isNotificationsOpen}
        onClose={() => setIsNotificationsOpen(false)}
      />

      {/* Command Palette (Cmd+Shift+K) */}
      {isCommandPaletteOpen && (
        <CommandPalette
          onClose={() => setIsCommandPaletteOpen(false)}
          onCommand={handleCommand}
          onNavigate={setActiveView}
          onOpenSearch={() => {
            setIsCommandPaletteOpen(false);
            setIsSearchOpen(true);
          }}
        />
      )}
    </div>
  );
}

export default App;
