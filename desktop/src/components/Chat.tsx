import { useState, useRef, useEffect } from 'react';
import { Conversation, Message } from '../App';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';
import remarkGfm from 'remark-gfm';
import './Chat.css';

interface ChatProps {
  conversation: Conversation | null;
  isProcessing: boolean;
  onSendMessage: (content: string) => void;
}

function Chat({ conversation, isProcessing, onSendMessage }: ChatProps) {
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const messages = conversation?.messages || [];

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(
        textareaRef.current.scrollHeight,
        200
      )}px`;
    }
  }, [input]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim() && !isProcessing) {
      onSendMessage(input);
      setInput('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  if (!conversation) {
    return (
      <div className="chat-container">
        <div className="welcome-screen">
          <div className="welcome-icon">
            <svg
              width="64"
              height="64"
              viewBox="0 0 64 64"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <circle cx="32" cy="32" r="30" stroke="currentColor" strokeWidth="2" />
              <path
                d="M20 32L28 40L44 24"
                stroke="currentColor"
                strokeWidth="3"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </div>
          <h2 className="welcome-title">Welcome to Helios CLI</h2>
          <p className="welcome-subtitle">
            Your AI-powered command line interface assistant
          </p>
          <div className="welcome-actions">
            <div className="action-card">
              <span className="action-icon">💬</span>
              <span className="action-text">Start a conversation</span>
            </div>
            <div className="action-card">
              <span className="action-icon">⌨️</span>
              <span className="action-text">Press Ctrl+K for commands</span>
            </div>
            <div className="action-card">
              <span className="action-icon">🚀</span>
              <span className="action-text">Ask me anything</span>
            </div>
          </div>
        </div>
        <InputArea
          input={input}
          setInput={setInput}
          onSubmit={handleSubmit}
          onKeyDown={handleKeyDown}
          isProcessing={isProcessing}
          textareaRef={textareaRef}
        />
      </div>
    );
  }

  return (
    <div className="chat-container">
      <div className="messages-container">
        {messages.length === 0 ? (
          <div className="empty-chat">
            <p className="empty-chat-hint">
              Start a conversation by typing a message below.
            </p>
          </div>
        ) : (
          messages.map((message) => (
            <MessageBubble key={message.id} message={message} />
          ))
        )}
        {isProcessing && (
          <div className="message assistant">
            <div className="message-avatar">H</div>
            <div className="message-content">
              <div className="typing-indicator">
                <span></span>
                <span></span>
                <span></span>
              </div>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      <InputArea
        input={input}
        setInput={setInput}
        onSubmit={handleSubmit}
        onKeyDown={handleKeyDown}
        isProcessing={isProcessing}
        textareaRef={textareaRef}
      />
    </div>
  );
}

interface InputAreaProps {
  input: string;
  setInput: (value: string) => void;
  onSubmit: (e: React.FormEvent) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  isProcessing: boolean;
  textareaRef: React.RefObject<HTMLTextAreaElement>;
}

function InputArea({
  input,
  setInput,
  onSubmit,
  onKeyDown,
  isProcessing,
  textareaRef,
}: InputAreaProps) {
  return (
    <div className="input-container">
      <form className="input-form" onSubmit={onSubmit}>
        <textarea
          ref={textareaRef}
          className="message-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={onKeyDown}
          placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
          disabled={isProcessing}
          rows={1}
        />
        <button
          type="submit"
          className="send-button"
          disabled={!input.trim() || isProcessing}
        >
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M22 2L11 13M22 2L15 22L11 13M22 2L2 9L11 13"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
      </form>
    </div>
  );
}

interface MessageBubbleProps {
  message: Message;
}

function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === 'user';
  const isSystem = message.role === 'system';

  return (
    <div className={`message ${message.role}`}>
      <div className="message-avatar">
        {isUser ? 'U' : isSystem ? 'S' : 'H'}
      </div>
      <div className="message-content">
        {isSystem ? (
          <div className="system-message">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {message.content}
            </ReactMarkdown>
          </div>
        ) : (
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={{
              code({ node, className, children, ...props }) {
                const match = /language-(\w+)/.exec(className || '');
                const isInline = !match;
                return isInline ? (
                  <code className={className} {...props}>
                    {children}
                  </code>
                ) : (
                  <SyntaxHighlighter
                    style={oneDark}
                    language={match ? match[1] : ''}
                    PreTag="div"
                    className="code-block"
                  >
                    {String(children).replace(/\n$/, '')}
                  </SyntaxHighlighter>
                );
              },
            }}
          >
            {message.content}
          </ReactMarkdown>
        )}
        <div className="message-timestamp">
          {formatTime(message.timestamp)}
        </div>
      </div>
    </div>
  );
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export default Chat;