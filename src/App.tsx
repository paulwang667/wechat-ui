/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { WindowFrame } from './components/WindowFrame';
import { Sidebar, TabType } from './components/Sidebar';
import { ChatList } from './components/ChatList';
import { ChatArea } from './components/ChatArea';
import { ContactList } from './components/ContactList';
import { StorageOptimizerView } from './components/StorageOptimizerView';
import { SyncDiagnosticsView } from './components/SyncDiagnosticsView';
import { RustArchitectureView } from './components/RustArchitectureView';
import { PerformanceBenchmarkView } from './components/PerformanceBenchmarkView';
import { MediaPreviewModal } from './components/MediaPreviewModal';
import { ProfileModal } from './components/ProfileModal';

import { 
  CURRENT_USER, 
  INITIAL_CONTACTS, 
  INITIAL_CONVERSATIONS, 
  INITIAL_MESSAGES 
} from './mock/initialData';
import { Contact, Conversation, Message, NetworkCondition } from './types/wechat';
import { rustStorageEngine } from './services/rustStorageEngine';
import { syncEngine } from './services/syncEngine';

export default function App() {
  const [currentUser, setCurrentUser] = useState<Contact>(CURRENT_USER);
  const [contacts, setContacts] = useState<Contact[]>(INITIAL_CONTACTS);
  const [conversations, setConversations] = useState<Conversation[]>(INITIAL_CONVERSATIONS);
  const [activeConvId, setActiveConvId] = useState<string>('conv_rust_bot');
  const [activeTab, setActiveTab] = useState<TabType>('chats');
  const [isDarkMode, setIsDarkMode] = useState<boolean>(true);
  const [previewImageUrl, setPreviewImageUrl] = useState<string | null>(null);
  const [isProfileModalOpen, setIsProfileModalOpen] = useState<boolean>(false);
  
  // Storage & Sync live state
  const [messagesVersion, setMessagesVersion] = useState<number>(0);
  const [networkCondition, setNetworkCondition] = useState<NetworkCondition>(syncEngine.getNetworkCondition());
  const [currentSeq, setCurrentSeq] = useState<number>(syncEngine.getCurrentSeq());
  const [outboxCount, setOutboxCount] = useState<number>(0);

  // Initialize storage engine with mock data
  useEffect(() => {
    rustStorageEngine.init(INITIAL_MESSAGES);
    setMessagesVersion(v => v + 1);
  }, []);

  // Listen to sync engine events
  useEffect(() => {
    const unsub = syncEngine.subscribe((event) => {
      if (event.type === 'message_saved' || event.type === 'incoming_message') {
        const msg = event.payload as Message;
        
        // Update conversation last message & unread count
        setConversations(prev => prev.map(c => {
          if (c.id === msg.conversationId) {
            const isUnread = !msg.isSelf && activeConvId !== msg.conversationId;
            return {
              ...c,
              lastMessage: msg.type === 'code' ? '[Code Snippet]' : msg.type === 'voice' ? '[Voice Note]' : msg.content,
              lastMessageTime: msg.timestamp,
              lastSyncSeq: msg.seqId,
              unreadCount: isUnread ? c.unreadCount + 1 : c.unreadCount,
            };
          }
          return c;
        }));

        setMessagesVersion(v => v + 1);
        setCurrentSeq(syncEngine.getCurrentSeq());
        setOutboxCount(syncEngine.getOutbox().length);
      } else if (event.type === 'message_status_changed') {
        setMessagesVersion(v => v + 1);
        setOutboxCount(syncEngine.getOutbox().length);
      } else if (event.type === 'network_changed') {
        setNetworkCondition(syncEngine.getNetworkCondition());
        setOutboxCount(syncEngine.getOutbox().length);
      } else if (event.type === 'sync_completed') {
        setCurrentSeq(syncEngine.getCurrentSeq());
      }
    });

    return () => unsub();
  }, [activeConvId]);

  // Sync Dark Mode class with root HTML
  useEffect(() => {
    if (isDarkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [isDarkMode]);

  // Messages for active conversation from local storage
  const activeConversation = useMemo(() => {
    return conversations.find(c => c.id === activeConvId) || null;
  }, [conversations, activeConvId]);

  const activeMessages = useMemo(() => {
    if (!activeConvId) return [];
    return rustStorageEngine.getMessagesByConversation(activeConvId);
  }, [activeConvId, messagesVersion]);

  const totalUnreadCount = useMemo(() => {
    return conversations.reduce((sum, c) => sum + c.unreadCount, 0);
  }, [conversations]);

  // Handler: Select Conversation
  const handleSelectConversation = useCallback((id: string) => {
    setActiveConvId(id);
    setActiveTab('chats');
    // Clear unread count on open
    setConversations(prev => prev.map(c => c.id === id ? { ...c, unreadCount: 0 } : c));
  }, []);

  // Handler: Send Message
  const handleSendMessage = useCallback(async (content: string, type: Message['type'] = 'text', extras?: Partial<Message>) => {
    if (!activeConvId) return;

    await syncEngine.sendMessage({
      conversationId: activeConvId,
      senderId: currentUser.id,
      senderName: currentUser.name,
      senderAvatar: currentUser.avatar,
      isSelf: true,
      type,
      content,
      timestamp: Date.now(),
      ...extras,
    });
  }, [activeConvId, currentUser]);

  // Handler: Retry Failed Message
  const handleRetryMessage = useCallback((msg: Message) => {
    syncEngine.flushOutbox();
  }, []);

  // Handler: Clear Messages in conversation
  const handleClearMessages = useCallback((convId: string) => {
    rustStorageEngine.clearConversation(convId);
    setConversations(prev => prev.map(c => c.id === convId ? { ...c, lastMessage: '', unreadCount: 0 } : c));
    setMessagesVersion(v => v + 1);
  }, []);

  // Handler: Toggle Pin
  const handleTogglePin = useCallback((id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setConversations(prev => prev.map(c => c.id === id ? { ...c, isPinned: !c.isPinned } : c));
  }, []);

  // Handler: Toggle Mute
  const handleToggleMute = useCallback((id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setConversations(prev => prev.map(c => c.id === id ? { ...c, isMuted: !c.isMuted } : c));
  }, []);

  // Handler: Start Chat with Contact
  const handleStartChatWithContact = useCallback((contact: Contact) => {
    // Check if conversation already exists
    let existing = conversations.find(c => c.id === `conv_${contact.id}` || c.name === contact.name);
    if (!existing) {
      const newConv: Conversation = {
        id: `conv_${contact.id}`,
        name: contact.name,
        avatar: contact.avatar,
        isGroup: false,
        lastMessage: 'Started chat with ' + contact.name,
        lastMessageTime: Date.now(),
        unreadCount: 0,
        isPinned: false,
        isMuted: false,
        lastSyncSeq: syncEngine.getCurrentSeq(),
      };
      setConversations(prev => [newConv, ...prev]);
      setActiveConvId(newConv.id);
    } else {
      setActiveConvId(existing.id);
    }
    setActiveTab('chats');
  }, [conversations]);

  // Handler: Stress test batch generation
  const handleStartStressTest = useCallback((count: number) => {
    const targetConvId = activeConvId || 'conv_rust_bot';
    rustStorageEngine.batchInsertStressTest(count, targetConvId);
    setMessagesVersion(v => v + 1);
    setCurrentSeq(syncEngine.getCurrentSeq() + count);
  }, [activeConvId]);

  return (
    <div className={`h-screen w-screen flex flex-col overflow-hidden font-sans select-none ${
      isDarkMode ? 'bg-[#18181c] text-zinc-100' : 'bg-[#e5e5e5] text-zinc-900'
    }`}>
      {/* Desktop Window Titlebar (macOS Style Frame) */}
      <WindowFrame
        network={networkCondition}
        isDarkMode={isDarkMode}
        onToggleDarkMode={() => setIsDarkMode(!isDarkMode)}
        onManualSync={() => syncEngine.triggerManualSync()}
        seqId={currentSeq}
        activeTab={activeTab}
      />

      {/* Main Desktop Workspace Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Primary Navigation Dock */}
        <Sidebar
          activeTab={activeTab}
          onSelectTab={(tab) => setActiveTab(tab)}
          currentUser={currentUser}
          unreadTotal={totalUnreadCount}
          outboxCount={outboxCount}
          onOpenProfile={() => setIsProfileModalOpen(true)}
          isDarkMode={isDarkMode}
        />

        {/* View Switcher based on Active Tab */}
        {activeTab === 'chats' && (
          <div className="flex-1 flex h-full overflow-hidden">
            {/* Conversation List Column */}
            <ChatList
              conversations={conversations}
              activeConversationId={activeConvId}
              onSelectConversation={handleSelectConversation}
              onTogglePin={handleTogglePin}
              onToggleMute={handleToggleMute}
              onClearHistory={handleClearMessages}
              onNewChat={() => setActiveTab('contacts')}
              isDarkMode={isDarkMode}
            />

            {/* Active Chat Conversation Feed */}
            <ChatArea
              conversation={activeConversation}
              messages={activeMessages}
              currentUser={currentUser}
              onSendMessage={handleSendMessage}
              onRetryMessage={handleRetryMessage}
              onClearMessages={handleClearMessages}
              isDarkMode={isDarkMode}
              onPreviewImage={(url) => setPreviewImageUrl(url)}
            />
          </div>
        )}

        {activeTab === 'contacts' && (
          <ContactList
            contacts={contacts}
            onStartChatWithContact={handleStartChatWithContact}
            isDarkMode={isDarkMode}
          />
        )}

        {activeTab === 'storage' && (
          <StorageOptimizerView
            isDarkMode={isDarkMode}
            onRefreshMetrics={() => setMessagesVersion(v => v + 1)}
            onStartStressTest={handleStartStressTest}
          />
        )}

        {activeTab === 'sync_diag' && (
          <SyncDiagnosticsView
            isDarkMode={isDarkMode}
            network={networkCondition}
            onNetworkChange={(cond) => syncEngine.setNetworkCondition(cond)}
            onTriggerManualSync={() => syncEngine.triggerManualSync()}
          />
        )}

        {activeTab === 'rust_arch' && (
          <RustArchitectureView
            isDarkMode={isDarkMode}
          />
        )}

        {activeTab === 'benchmarks' && (
          <PerformanceBenchmarkView
            isDarkMode={isDarkMode}
          />
        )}
      </div>

      {/* Modals & Overlays */}
      <MediaPreviewModal
        imageUrl={previewImageUrl}
        onClose={() => setPreviewImageUrl(null)}
        isDarkMode={isDarkMode}
      />

      <ProfileModal
        isOpen={isProfileModalOpen}
        onClose={() => setIsProfileModalOpen(false)}
        currentUser={currentUser}
        onUpdateUser={(updated) => setCurrentUser(prev => ({ ...prev, ...updated }))}
        isDarkMode={isDarkMode}
      />
    </div>
  );
}
