import React, { useState, useMemo } from 'react';
import { 
  Search, 
  Pin, 
  BellOff, 
  Plus, 
  MessageSquarePlus, 
  CheckCheck,
  Trash2,
  Filter
} from 'lucide-react';
import { Conversation } from '../types/wechat';

interface ChatListProps {
  conversations: Conversation[];
  activeConversationId: string | null;
  onSelectConversation: (id: string) => void;
  onTogglePin: (id: string, e: React.MouseEvent) => void;
  onToggleMute: (id: string, e: React.MouseEvent) => void;
  onClearHistory: (id: string, e: React.MouseEvent) => void;
  onNewChat: () => void;
  isDarkMode: boolean;
}

export const ChatList: React.FC<ChatListProps> = ({
  conversations,
  activeConversationId,
  onSelectConversation,
  onTogglePin,
  onToggleMute,
  onClearHistory,
  onNewChat,
  isDarkMode,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'unread' | 'pinned'>('all');

  const filteredConversations = useMemo(() => {
    return conversations
      .filter((conv) => {
        if (filterType === 'unread' && conv.unreadCount === 0) return false;
        if (filterType === 'pinned' && !conv.isPinned) return false;
        if (!searchQuery) return true;
        return (
          conv.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          (conv.lastMessage && conv.lastMessage.toLowerCase().includes(searchQuery.toLowerCase()))
        );
      })
      .sort((a, b) => {
        // Pinned first, then sorted by lastMessageTime descending
        if (a.isPinned && !b.isPinned) return -1;
        if (!a.isPinned && b.isPinned) return 1;
        return b.lastMessageTime - a.lastMessageTime;
      });
  }, [conversations, searchQuery, filterType]);

  const formatTimestamp = (ts: number) => {
    const now = Date.now();
    const diff = now - ts;
    if (diff < 1000 * 60) return 'Just now';
    if (diff < 1000 * 3600) return `${Math.floor(diff / (1000 * 60))}m`;
    if (diff < 1000 * 86400) {
      const date = new Date(ts);
      return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
    }
    const date = new Date(ts);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  };

  return (
    <div
      id="wechat_chat_list_panel"
      className={`w-72 flex flex-col border-r select-none h-full transition-colors ${
        isDarkMode 
          ? 'bg-[#25252b] border-zinc-800 text-zinc-200' 
          : 'bg-[#ebecee] border-zinc-200 text-zinc-800'
      }`}
    >
      {/* Top Search & Filter Bar */}
      <div className="p-2.5 pb-2 space-y-2 border-b border-black/5 dark:border-white/5">
        <div className="flex items-center space-x-1.5">
          <div className={`flex-1 flex items-center px-2.5 py-1.5 rounded-md text-xs transition-colors ${
            isDarkMode 
              ? 'bg-[#18181c] text-zinc-200 focus-within:ring-1 focus-within:ring-emerald-500' 
              : 'bg-[#dcdcdc] text-zinc-800 focus-within:ring-1 focus-within:ring-emerald-500'
          }`}>
            <Search className="w-3.5 h-3.5 text-zinc-400 mr-2 shrink-0" />
            <input
              id="input_search_conversations"
              type="text"
              placeholder="Search chats & messages..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-transparent border-none outline-none text-xs placeholder-zinc-400"
            />
          </div>

          <button
            id="btn_create_new_chat"
            onClick={onNewChat}
            title="Start New Chat"
            className="p-1.5 rounded-md hover:bg-black/5 dark:hover:bg-white/10 text-zinc-600 dark:text-zinc-300 transition-colors"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>

        {/* Quick Filter Tabs */}
        <div className="flex items-center space-x-1 text-[11px]">
          {(['all', 'unread', 'pinned'] as const).map((type) => (
            <button
              key={type}
              onClick={() => setFilterType(type)}
              className={`px-2 py-0.5 rounded capitalize font-medium transition-all ${
                filterType === type
                  ? 'bg-emerald-600 text-white shadow-xs'
                  : 'text-zinc-500 dark:text-zinc-400 hover:bg-black/5 dark:hover:bg-white/5'
              }`}
            >
              {type}
            </button>
          ))}
        </div>
      </div>

      {/* Conversation List */}
      <div className="flex-1 overflow-y-auto divide-y divide-black/5 dark:divide-white/5">
        {filteredConversations.length === 0 ? (
          <div className="py-12 px-4 text-center text-xs text-zinc-400">
            <MessageSquarePlus className="w-8 h-8 mx-auto mb-2 text-zinc-300 dark:text-zinc-600" />
            <p>No conversations found</p>
          </div>
        ) : (
          filteredConversations.map((conv) => {
            const isActive = activeConversationId === conv.id;

            return (
              <div
                key={conv.id}
                id={`chat_item_${conv.id}`}
                onClick={() => onSelectConversation(conv.id)}
                className={`relative px-3 py-2.5 flex items-start space-x-3 cursor-pointer group transition-all ${
                  isActive
                    ? isDarkMode 
                      ? 'bg-[#18181c]' 
                      : 'bg-[#c7c9cc]'
                    : conv.isPinned
                    ? isDarkMode
                      ? 'bg-[#2c2c34] hover:bg-[#34343d]'
                      : 'bg-[#e2e4e7] hover:bg-[#d8dade]'
                    : isDarkMode
                    ? 'hover:bg-[#2e2e36]'
                    : 'hover:bg-[#e4e5e8]'
                }`}
              >
                {/* Pin badge indicator */}
                {conv.isPinned && (
                  <span className="absolute top-1.5 right-1.5 text-zinc-400">
                    <Pin className="w-2.5 h-2.5 fill-current rotate-45" />
                  </span>
                )}

                {/* Avatar */}
                <div className="relative shrink-0 mt-0.5">
                  <img
                    src={conv.avatar}
                    alt={conv.name}
                    className="w-10 h-10 rounded-lg object-cover shadow-xs"
                  />
                  {conv.unreadCount > 0 && (
                    <span className="absolute -top-1.5 -right-1.5 min-w-[17px] h-[17px] px-1 text-[10px] font-bold text-white bg-rose-500 rounded-full flex items-center justify-center border-2 border-white dark:border-[#25252b] shadow">
                      {conv.unreadCount > 99 ? '99+' : conv.unreadCount}
                    </span>
                  )}
                </div>

                {/* Content & Metadata */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between">
                    <h4 className="text-xs font-semibold truncate text-zinc-900 dark:text-zinc-100 pr-1">
                      {conv.name}
                    </h4>
                    <span className="text-[10px] text-zinc-400 shrink-0 font-mono">
                      {formatTimestamp(conv.lastMessageTime)}
                    </span>
                  </div>

                  <div className="flex items-center justify-between mt-1">
                    <p className="text-[11px] text-zinc-500 dark:text-zinc-400 truncate pr-2">
                      {conv.lastMessage || 'No messages yet'}
                    </p>
                    {conv.isMuted && (
                      <BellOff className="w-3 h-3 text-zinc-400 shrink-0" />
                    )}
                  </div>
                </div>

                {/* Hover Quick Actions */}
                <div className="absolute right-2 bottom-1.5 hidden group-hover:flex items-center space-x-1 bg-white/90 dark:bg-zinc-800/90 backdrop-blur-xs px-1.5 py-0.5 rounded shadow-sm">
                  <button
                    onClick={(e) => onTogglePin(conv.id, e)}
                    title={conv.isPinned ? 'Unpin Chat' : 'Pin Chat'}
                    className="p-1 hover:text-emerald-500 transition-colors"
                  >
                    <Pin className={`w-3 h-3 ${conv.isPinned ? 'fill-emerald-500 text-emerald-500' : ''}`} />
                  </button>
                  <button
                    onClick={(e) => onToggleMute(conv.id, e)}
                    title={conv.isMuted ? 'Unmute' : 'Mute'}
                    className="p-1 hover:text-amber-500 transition-colors"
                  >
                    <BellOff className={`w-3 h-3 ${conv.isMuted ? 'text-amber-500' : ''}`} />
                  </button>
                  <button
                    onClick={(e) => onClearHistory(conv.id, e)}
                    title="Clear Chat Messages"
                    className="p-1 hover:text-rose-500 transition-colors"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Bottom status bar */}
      <div className="p-2 border-t border-black/5 dark:border-white/5 text-[10px] text-zinc-400 flex items-center justify-between font-mono">
        <span>{filteredConversations.length} Conversations</span>
        <span className="text-emerald-600 dark:text-emerald-400 flex items-center space-x-1">
          <CheckCheck className="w-3 h-3" />
          <span>Local Store Ready</span>
        </span>
      </div>
    </div>
  );
};
