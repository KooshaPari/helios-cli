import { useState, useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type { SearchResult, SearchQuery, ToolSource, ResultType } from '../types';
import './UnifiedSearch.css';

interface UnifiedSearchProps {
  isOpen: boolean;
  onClose: () => void;
  onNavigateToResult?: (url: string) => void;
}

const SOURCE_LABELS: Record<ToolSource, string> = {
  tracera: 'Tracera',
  agileplus: 'AgilePlus',
  github: 'GitHub',
  helios: 'Helios',
};

const SOURCE_COLORS: Record<ToolSource, string> = {
  tracera: '#3b82f6',
  agileplus: '#a855f7',
  github: '#e6edf3',
  helios: '#10b981',
};

const TYPE_ICONS: Record<ResultType, string> = {
  issue: '\u26a0',
  task: '\u25a0',
  spec: '\u2606',
  pull_request: '\u2192',
  workflow: '\u2699',
  notification: '\u266b',
  agent: '\u25cf',
};

function UnifiedSearch({ isOpen, onClose, onNavigateToResult }: UnifiedSearchProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [sourceFilter, setSourceFilter] = useState<ToolSource | null>(null);
  const [typeFilter, setTypeFilter] = useState<ResultType | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setResults([]);
      setSelectedIndex(0);
      // Focus the input after a small delay to ensure the overlay is rendered.
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const performSearch = useCallback(async (text: string) => {
    if (!text.trim()) {
      setResults([]);
      return;
    }
    setLoading(true);
    try {
      const searchQuery: SearchQuery = {
        text: text.trim(),
        source_filter: sourceFilter,
        type_filter: typeFilter,
      };
      const searchResults = await invoke<SearchResult[]>('unified_search_cmd', {
        query: searchQuery,
      });
      setResults(searchResults);
      setSelectedIndex(0);
    } catch (err) {
      console.error('Search failed:', err);
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, [sourceFilter, typeFilter]);

  const handleQueryChange = (value: string) => {
    setQuery(value);
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    debounceRef.current = setTimeout(() => {
      performSearch(value);
    }, 200);
  };

  // Re-search when filters change.
  useEffect(() => {
    if (query.trim()) {
      performSearch(query);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sourceFilter, typeFilter]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev < results.length - 1 ? prev + 1 : 0
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev > 0 ? prev - 1 : results.length - 1
        );
        break;
      case 'Enter':
        e.preventDefault();
        if (results[selectedIndex]) {
          handleSelectResult(results[selectedIndex]);
        }
        break;
      case 'Escape':
        onClose();
        break;
    }
  };

  const handleSelectResult = (result: SearchResult) => {
    if (result.url && onNavigateToResult) {
      onNavigateToResult(result.url);
    }
    onClose();
  };

  const toggleSourceFilter = (source: ToolSource) => {
    setSourceFilter((prev) => (prev === source ? null : source));
  };

  if (!isOpen) return null;

  return (
    <div className="search-overlay" onClick={onClose}>
      <div className="search-panel" onClick={(e) => e.stopPropagation()}>
        {/* Search input */}
        <div className="search-header">
          <span className="search-icon">\u2318K</span>
          <input
            ref={inputRef}
            className="search-input"
            type="text"
            value={query}
            onChange={(e) => handleQueryChange(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search across Tracera, AgilePlus, and Helios..."
          />
          {loading && <span className="search-spinner" />}
        </div>

        {/* Source filter badges */}
        <div className="search-filters">
          {(Object.keys(SOURCE_LABELS) as ToolSource[]).map((source) => (
            <button
              key={source}
              className={`filter-badge ${sourceFilter === source ? 'active' : ''}`}
              style={{
                '--badge-color': SOURCE_COLORS[source],
              } as React.CSSProperties}
              onClick={() => toggleSourceFilter(source)}
            >
              <span
                className="filter-dot"
                style={{ background: SOURCE_COLORS[source] }}
              />
              {SOURCE_LABELS[source]}
            </button>
          ))}
          {(sourceFilter || typeFilter) && (
            <button
              className="filter-clear"
              onClick={() => {
                setSourceFilter(null);
                setTypeFilter(null);
              }}
            >
              Clear filters
            </button>
          )}
        </div>

        <div className="search-divider" />

        {/* Results */}
        <div className="search-results">
          {results.length === 0 && query.trim() && !loading && (
            <div className="search-no-results">No results found for &ldquo;{query}&rdquo;</div>
          )}
          {results.map((result, index) => (
            <div
              key={`${result.source}-${result.title}-${index}`}
              className={`search-result ${index === selectedIndex ? 'selected' : ''}`}
              onClick={() => handleSelectResult(result)}
              onMouseEnter={() => setSelectedIndex(index)}
            >
              <span
                className="result-source-badge"
                style={{ background: SOURCE_COLORS[result.source] }}
              >
                {SOURCE_LABELS[result.source]}
              </span>
              <span className="result-type-icon">
                {TYPE_ICONS[result.type] || '?'}
              </span>
              <div className="result-content">
                <div className="result-title">{result.title}</div>
                <div className="result-snippet">{result.snippet}</div>
              </div>
              {result.timestamp && (
                <span className="result-time">
                  {new Date(result.timestamp).toLocaleDateString()}
                </span>
              )}
            </div>
          ))}
        </div>

        {/* Footer hints */}
        <div className="search-footer">
          <span className="footer-hint">
            <kbd>\u2191\u2193</kbd> Navigate
          </span>
          <span className="footer-hint">
            <kbd>\u21b5</kbd> Open
          </span>
          <span className="footer-hint">
            <kbd>esc</kbd> Close
          </span>
        </div>
      </div>
    </div>
  );
}

export default UnifiedSearch;
