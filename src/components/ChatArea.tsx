import React, { useState, useRef, useEffect } from 'react';
import { 
  Smile, 
  Paperclip, 
  Image as ImageIcon, 
  Code, 
  Mic, 
  Send, 
  Phone, 
  Video, 
  MoreHorizontal, 
  Search, 
  Play, 
  Pause, 
  FileText, 
  Download, 
  Copy, 
  Check, 
  Clock, 
  AlertCircle, 
  Quote, 
  X,
  Volume2,
  Trash2,
  Cpu
} from 'lucide-react';
import { Conversation, Message, Contact } from '../types/wechat';

interface ChatAreaProps {
  conversation: Conversation | null;
  messages: Message[];
  currentUser: Contact;
  onSendMessage: (content: string, type?: Message['type'], extras?: Partial<Message>) => void;
  onRetryMessage?: (msg: Message) => void;
  onClearMessages: (convId: string) => void;
  isDarkMode: boolean;
  onPreviewImage: (url: string) => void;
}

const COMMON_EMOJIS = ['👍', '🎉', '🦀', '🔥', '🚀', '❤️', '😄', '🙏', '⚡', '☕', '💡', '💯', '👏', '🤖', '📦'];

export const ChatArea: React.FC<ChatAreaProps> = ({
  conversation,
  messages,
  currentUser,
  onSendMessage,
  onRetryMessage,
  onClearMessages,
  isDarkMode,
  onPreviewImage,
}) => {
  const [inputText, setInputText] = useState('');
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const [showCodeModal, setShowCodeModal] = useState(false);
  const [codeLang, setCodeLang] = useState('rust');
  const [codeContent, setCodeContent] = useState('');
  const [isRecordingVoice, setIsRecordingVoice] = useState(false);
  const [recordingSeconds, setRecordingSeconds] = useState(0);
  const [playingVoiceId, setPlayingVoiceId] = useState<string | null>(null);
  const [copiedCodeId, setCopiedCodeId] = useState<string | null>(null);
  const [replyingTo, setReplyingTo] = useState<Message | null>(null);
  const [inChatSearch, setInChatSearch] = useState('');
  const [showInChatSearch, setShowInChatSearch] = useState(false);
  const [showDetailsDrawer, setShowDetailsDrawer] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const voiceTimerRef = useRef<any>(null);

  // Auto scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages.length, conversation?.id]);

  // Voice recording timer simulation
  useEffect(() => {
    if (isRecordingVoice) {
      setRecordingSeconds(1);
      voiceTimerRef.current = setInterval(() => {
        setRecordingSeconds((prev) => {
          if (prev >= 15) {
            handleStopAndSendVoice();
            return 0;
          }
          return prev + 1;
        });
      }, 1000);
    } else {
      if (voiceTimerRef.current) clearInterval(voiceTimerRef.current);
      setRecordingSeconds(0);
    }
    return () => {
      if (voiceTimerRef.current) clearInterval(voiceTimerRef.current);
    };
  }, [isRecordingVoice]);

  const handleSend = () => {
    if (!inputText.trim()) return;
    const extras: Partial<Message> = {};
    if (replyingTo) {
      extras.quotedMessage = {
        senderName: replyingTo.senderName,
        content: replyingTo.content.slice(0, 80),
      };
      setReplyingTo(null);
    }
    onSendMessage(inputText.trim(), 'text', extras);
    setInputText('');
    setShowEmojiPicker(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleSendCodeSnippet = () => {
    if (!codeContent.trim()) return;
    onSendMessage(
      codeContent.trim().slice(0, 60) + '...',
      'code',
      {
        codeLanguage: codeLang,
        codeSnippet: codeContent.trim(),
      }
    );
    setCodeContent('');
    setShowCodeModal(false);
  };

  const handleStopAndSendVoice = () => {
    setIsRecordingVoice(false);
    const duration = Math.max(1, recordingSeconds);
    onSendMessage(`[Voice Memo: ${duration}s]`, 'voice', {
      voiceDuration: duration,
    });
  };

  const handleSendSampleImage = () => {
    const sampleImages = [
      'https://images.unsplash.com/photo-1555066931-4365d14bab8c?w=600&auto=format&fit=crop&q=80',
      'https://images.unsplash.com/photo-1518770660439-4636190af475?w=600&auto=format&fit=crop&q=80',
      'https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?w=600&auto=format&fit=crop&q=80',
    ];
    const picked = sampleImages[Math.floor(Math.random() * sampleImages.length)];
    onSendMessage('[Image]', 'image', {
      mediaUrl: picked,
    });
  };

  const handleSendSampleFile = () => {
    const sampleFiles = [
      { name: 'rust_fenestra_wechat_arch.pdf', size: '2.8 MB' },
      { name: 'tokio_sync_benchmarks.json', size: '340 KB' },
      { name: 'sqlite_wal_compaction_v2.rs', size: '18 KB' },
    ];
    const picked = sampleFiles[Math.floor(Math.random() * sampleFiles.length)];
    onSendMessage(picked.name, 'file', {
      fileName: picked.name,
      fileSize: picked.size,
    });
  };

  const handleCopyCode = (id: string, codeText: string) => {
    navigator.clipboard.writeText(codeText);
    setCopiedCodeId(id);
    setTimeout(() => setCopiedCodeId(null), 2000);
  };

  const togglePlayVoice = (id: string) => {
    if (playingVoiceId === id) {
      setPlayingVoiceId(null);
    } else {
      setPlayingVoiceId(id);
      // auto stop after 3s
      setTimeout(() => setPlayingVoiceId(null), 3000);
    }
  };

  if (!conversation) {
    return (
      <div className={`flex-1 flex flex-col items-center justify-center select-none ${
        isDarkMode ? 'bg-[#18181c] text-zinc-500' : 'bg-[#f5f5f5] text-zinc-400'
      }`}>
        <div className="w-16 h-16 rounded-2xl bg-emerald-500/10 flex items-center justify-center text-emerald-500 mb-3">
          <Cpu className="w-8 h-8" />
        </div>
        <h3 className="text-sm font-semibold text-zinc-700 dark:text-zinc-300">WeChat Desktop (Rust Native)</h3>
        <p className="text-xs mt-1">Select a conversation to inspect sync sequences & storage compaction</p>
      </div>
    );
  }

  const displayedMessages = inChatSearch
    ? messages.filter((m) => m.content.toLowerCase().includes(inChatSearch.toLowerCase()))
    : messages;

  return (
    <div
      id="wechat_chat_area_panel"
      className={`flex-1 flex flex-col h-full overflow-hidden transition-colors ${
        isDarkMode ? 'bg-[#1e1e24] text-zinc-200' : 'bg-[#f5f5f5] text-zinc-800'
      }`}
    >
      {/* Chat Header */}
      <div className={`h-14 px-4 flex items-center justify-between border-b select-none ${
        isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-[#f3f4f6] border-zinc-200'
      }`}>
        <div className="flex items-center space-x-3">
          <img
            src={conversation.avatar}
            alt={conversation.name}
            className="w-9 h-9 rounded-lg object-cover shadow-xs"
          />
          <div>
            <div className="flex items-center space-x-2">
              <h3 className="text-xs font-bold text-zinc-900 dark:text-zinc-100">
                {conversation.name}
              </h3>
              {conversation.isGroup && (
                <span className="text-[10px] px-1.5 py-0.2 rounded bg-zinc-200 dark:bg-zinc-700 text-zinc-600 dark:text-zinc-300 font-mono">
                  Group (4)
                </span>
              )}
            </div>
            <p className="text-[10px] text-zinc-400 font-mono flex items-center space-x-1 mt-0.5">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 inline-block" />
              <span>Rust Sync Sequence: #{conversation.lastSyncSeq}</span>
            </p>
          </div>
        </div>

        {/* Right Header Actions */}
        <div className="flex items-center space-x-1">
          <button
            id="btn_chat_search_toggle"
            onClick={() => setShowInChatSearch(!showInChatSearch)}
            title="Search in this chat"
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 text-zinc-600 dark:text-zinc-300 transition-colors"
          >
            <Search className="w-4 h-4" />
          </button>

          <button
            onClick={() => alert(`Calling ${conversation.name} via Rust WebRTC audio bridge simulation...`)}
            title="Audio Call"
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 text-zinc-600 dark:text-zinc-300 transition-colors"
          >
            <Phone className="w-4 h-4" />
          </button>

          <button
            onClick={() => alert(`Video Call with ${conversation.name} simulated.`)}
            title="Video Call"
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 text-zinc-600 dark:text-zinc-300 transition-colors"
          >
            <Video className="w-4 h-4" />
          </button>

          <button
            id="btn_chat_details_toggle"
            onClick={() => setShowDetailsDrawer(!showDetailsDrawer)}
            title="Conversation Details & Storage"
            className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 text-zinc-600 dark:text-zinc-300 transition-colors"
          >
            <MoreHorizontal className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* In-chat search bar toggle */}
      {showInChatSearch && (
        <div className={`px-4 py-2 flex items-center space-x-2 border-b text-xs ${
          isDarkMode ? 'bg-[#18181c] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <Search className="w-3.5 h-3.5 text-zinc-400 shrink-0" />
          <input
            type="text"
            placeholder="Search within this conversation..."
            value={inChatSearch}
            onChange={(e) => setInChatSearch(e.target.value)}
            className="flex-1 bg-transparent border-none outline-none text-xs"
            autoFocus
          />
          {inChatSearch && (
            <button onClick={() => setInChatSearch('')} className="text-zinc-400 hover:text-zinc-600">
              <X className="w-3 h-3" />
            </button>
          )}
          <span className="text-[11px] text-zinc-400 font-mono">
            {displayedMessages.length} found
          </span>
        </div>
      )}

      {/* Message Feed */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {displayedMessages.length === 0 ? (
          <div className="py-20 text-center text-xs text-zinc-400">
            No messages found. Send a message to test zero-copy incremental synchronization!
          </div>
        ) : (
          displayedMessages.map((msg, index) => {
            const isSelf = msg.isSelf;
            const showDateHeader = index === 0 || (msg.timestamp - displayedMessages[index - 1].timestamp > 1000 * 60 * 15);

            return (
              <div key={msg.id} className="space-y-2">
                {/* Date header */}
                {showDateHeader && (
                  <div className="flex justify-center my-2">
                    <span className="px-2.5 py-0.5 rounded-full text-[10px] font-mono bg-black/5 dark:bg-white/5 text-zinc-400">
                      {new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </span>
                  </div>
                )}

                {/* System Message */}
                {msg.type === 'system' ? (
                  <div className="flex justify-center my-2">
                    <div className="max-w-md px-3 py-1.5 rounded-md bg-emerald-500/10 border border-emerald-500/20 text-emerald-700 dark:text-emerald-300 text-[11px] text-center font-mono">
                      {msg.content}
                    </div>
                  </div>
                ) : (
                  /* Standard / Rich Message */
                  <div className={`flex items-start space-x-2.5 ${isSelf ? 'flex-row-reverse space-x-reverse' : ''}`}>
                    {/* Avatar */}
                    <img
                      src={msg.senderAvatar}
                      alt={msg.senderName}
                      className="w-9 h-9 rounded-lg object-cover shadow-xs shrink-0 mt-0.5"
                    />

                    {/* Bubble Container */}
                    <div className={`max-w-[72%] space-y-1 ${isSelf ? 'items-end' : 'items-start'} flex flex-col`}>
                      {/* Sender Name in group */}
                      {!isSelf && (
                        <span className="text-[11px] text-zinc-400 font-medium px-1">
                          {msg.senderName}
                        </span>
                      )}

                      {/* Quoted preview if present */}
                      {msg.quotedMessage && (
                        <div className={`text-[11px] px-2.5 py-1 rounded border-l-2 bg-black/5 dark:bg-white/5 border-emerald-500 text-zinc-500 dark:text-zinc-400 truncate mb-1`}>
                          <span className="font-semibold">{msg.quotedMessage.senderName}: </span>
                          <span>{msg.quotedMessage.content}</span>
                        </div>
                      )}

                      {/* Bubble Content by Type */}
                      <div className="relative group">
                        {/* 1. Plain Text */}
                        {msg.type === 'text' && (
                          <div
                            className={`px-3.5 py-2 rounded-2xl text-xs leading-relaxed break-words shadow-xs select-text ${
                              isSelf
                                ? 'bg-[#95ec69] text-zinc-900 rounded-tr-xs dark:bg-emerald-600 dark:text-white'
                                : isDarkMode
                                ? 'bg-[#2a2a32] text-zinc-100 rounded-tl-xs'
                                : 'bg-white text-zinc-800 rounded-tl-xs'
                            }`}
                          >
                            {msg.content}
                          </div>
                        )}

                        {/* 2. Code Snippet */}
                        {msg.type === 'code' && (
                          <div className={`rounded-xl border overflow-hidden shadow-sm text-xs ${
                            isDarkMode ? 'bg-[#18181c] border-zinc-800' : 'bg-[#282c34] border-zinc-700 text-zinc-100'
                          }`}>
                            <div className="px-3 py-1.5 bg-black/20 flex items-center justify-between border-b border-white/10 text-[11px] font-mono text-zinc-400">
                              <span className="font-bold text-emerald-400">{msg.codeLanguage || 'rust'}</span>
                              <button
                                onClick={() => handleCopyCode(msg.id, msg.codeSnippet || msg.content)}
                                className="flex items-center space-x-1 hover:text-white transition-colors"
                              >
                                {copiedCodeId === msg.id ? (
                                  <>
                                    <Check className="w-3 h-3 text-emerald-400" />
                                    <span className="text-emerald-400">Copied</span>
                                  </>
                                ) : (
                                  <>
                                    <Copy className="w-3 h-3" />
                                    <span>Copy</span>
                                  </>
                                )}
                              </button>
                            </div>
                            <pre className="p-3 font-mono text-[11px] overflow-x-auto text-emerald-300 leading-normal">
                              <code>{msg.codeSnippet || msg.content}</code>
                            </pre>
                          </div>
                        )}

                        {/* 3. Voice Memo */}
                        {msg.type === 'voice' && (
                          <button
                            onClick={() => togglePlayVoice(msg.id)}
                            className={`px-3.5 py-2.5 rounded-2xl flex items-center space-x-3 text-xs shadow-xs transition-all ${
                              isSelf
                                ? 'bg-[#95ec69] text-zinc-900 rounded-tr-xs dark:bg-emerald-600 dark:text-white'
                                : isDarkMode
                                ? 'bg-[#2a2a32] text-zinc-100 rounded-tl-xs'
                                : 'bg-white text-zinc-800 rounded-tl-xs'
                            }`}
                          >
                            <div className="w-6 h-6 rounded-full bg-black/10 dark:bg-white/10 flex items-center justify-center">
                              {playingVoiceId === msg.id ? (
                                <Pause className="w-3.5 h-3.5 animate-pulse" />
                              ) : (
                                <Play className="w-3.5 h-3.5 ml-0.5" />
                              )}
                            </div>

                            {/* Waveform graphic */}
                            <div className="flex items-center space-x-0.5 h-4">
                              {[12, 18, 8, 22, 14, 20, 10, 16].map((h, i) => (
                                <span
                                  key={i}
                                  style={{ height: `${playingVoiceId === msg.id ? (h + (i % 3) * 3) : h}px` }}
                                  className={`w-0.5 rounded-full transition-all ${
                                    isSelf ? 'bg-zinc-800 dark:bg-white' : 'bg-emerald-500'
                                  }`}
                                />
                              ))}
                            </div>

                            <span className="font-mono text-[11px] font-bold">
                              {msg.voiceDuration || 3}"
                            </span>
                          </button>
                        )}

                        {/* 4. Image */}
                        {msg.type === 'image' && msg.mediaUrl && (
                          <div className="rounded-xl overflow-hidden shadow-md cursor-pointer group/img">
                            <img
                              src={msg.mediaUrl}
                              alt="attachment"
                              onClick={() => onPreviewImage(msg.mediaUrl!)}
                              className="max-w-[280px] max-h-[200px] object-cover hover:scale-102 transition-transform duration-200"
                            />
                          </div>
                        )}

                        {/* 5. File Attachment */}
                        {msg.type === 'file' && (
                          <div className={`p-3 rounded-xl border flex items-center space-x-3 shadow-xs text-xs ${
                            isDarkMode ? 'bg-[#25252b] border-zinc-700' : 'bg-white border-zinc-200'
                          }`}>
                            <div className="w-10 h-10 rounded-lg bg-emerald-500/10 text-emerald-500 flex items-center justify-center shrink-0">
                              <FileText className="w-5 h-5" />
                            </div>
                            <div className="min-w-0 flex-1">
                              <p className="font-medium truncate text-zinc-800 dark:text-zinc-100">
                                {msg.fileName || 'file_attachment.bin'}
                              </p>
                              <p className="text-[10px] text-zinc-400 font-mono">
                                {msg.fileSize || '1.2 MB'}
                              </p>
                            </div>
                            <button
                              onClick={() => alert(`Downloading ${msg.fileName} from Rust local blob cache...`)}
                              className="p-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 text-zinc-500 hover:text-emerald-500 transition-colors"
                              title="Download File"
                            >
                              <Download className="w-4 h-4" />
                            </button>
                          </div>
                        )}

                        {/* Quick Quote trigger on hover */}
                        <div className={`absolute top-0 hidden group-hover:flex items-center space-x-1 ${
                          isSelf ? '-left-12' : '-right-12'
                        }`}>
                          <button
                            onClick={() => setReplyingTo(msg)}
                            title="Quote / Reply"
                            className="p-1 rounded bg-black/10 dark:bg-white/10 hover:bg-emerald-500 hover:text-white text-zinc-500 transition-colors"
                          >
                            <Quote className="w-3 h-3" />
                          </button>
                        </div>
                      </div>

                      {/* Footer Metadata: Sequence number, Compression size, Sync state */}
                      <div className={`flex items-center space-x-1.5 text-[9px] font-mono text-zinc-400 px-1 ${
                        isSelf ? 'justify-end' : 'justify-start'
                      }`}>
                        <span>Seq #{msg.seqId}</span>
                        {msg.compressedBytes && (
                          <>
                            <span>•</span>
                            <span title={`Raw: ${msg.rawBytes}B, Compressed: ${msg.compressedBytes}B`}>
                              {msg.compressedBytes}B LZ4
                            </span>
                          </>
                        )}
                        {isSelf && (
                          <>
                            {msg.status === 'sending' && <Clock className="w-2.5 h-2.5 text-amber-500 animate-spin" />}
                            {msg.status === 'pending' && <span className="text-amber-500">[Outbox]</span>}
                            {msg.status === 'failed' && (
                              <button
                                onClick={() => onRetryMessage && onRetryMessage(msg)}
                                className="flex items-center space-x-0.5 text-rose-500 hover:underline cursor-pointer"
                              >
                                <AlertCircle className="w-2.5 h-2.5" />
                                <span>Retry</span>
                              </button>
                            )}
                            {msg.status === 'synced' && <Check className="w-2.5 h-2.5 text-emerald-500" />}
                          </>
                        )}
                      </div>
                    </div>
                  </div>
                )}
              </div>
            );
          })
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Replying Banner */}
      {replyingTo && (
        <div className={`px-4 py-1.5 flex items-center justify-between border-t text-xs ${
          isDarkMode ? 'bg-[#18181c] border-zinc-800 text-zinc-300' : 'bg-zinc-100 border-zinc-200 text-zinc-700'
        }`}>
          <div className="flex items-center space-x-2 truncate">
            <Quote className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
            <span className="font-semibold">{replyingTo.senderName}:</span>
            <span className="text-zinc-500 truncate">{replyingTo.content}</span>
          </div>
          <button
            onClick={() => setReplyingTo(null)}
            className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-200"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Input Toolbar & Area */}
      <div className={`border-t select-none ${
        isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
      }`}>
        {/* Toolbar Icons */}
        <div className="px-3 py-1.5 flex items-center justify-between">
          <div className="flex items-center space-x-1 text-zinc-500 dark:text-zinc-400">
            {/* Emoji Trigger */}
            <div className="relative">
              <button
                id="btn_toolbar_emoji"
                onClick={() => setShowEmojiPicker(!showEmojiPicker)}
                title="Emojis"
                className="p-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 hover:text-emerald-500 transition-colors"
              >
                <Smile className="w-4 h-4" />
              </button>

              {/* Emoji Picker Popover */}
              {showEmojiPicker && (
                <div className={`absolute bottom-8 left-0 p-2 rounded-xl shadow-xl border grid grid-cols-5 gap-1.5 z-20 w-48 ${
                  isDarkMode ? 'bg-[#1e1e24] border-zinc-700' : 'bg-white border-zinc-200'
                }`}>
                  {COMMON_EMOJIS.map((emoji) => (
                    <button
                      key={emoji}
                      onClick={() => {
                        setInputText((prev) => prev + emoji);
                        setShowEmojiPicker(false);
                        textareaRef.current?.focus();
                      }}
                      className="w-8 h-8 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 flex items-center justify-center text-base transition-transform hover:scale-110"
                    >
                      {emoji}
                    </button>
                  ))}
                </div>
              )}
            </div>

            {/* Image attachment */}
            <button
              id="btn_toolbar_image"
              onClick={handleSendSampleImage}
              title="Send Image (Zero-Copy Blob Hash)"
              className="p-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 hover:text-emerald-500 transition-colors"
            >
              <ImageIcon className="w-4 h-4" />
            </button>

            {/* File attachment */}
            <button
              id="btn_toolbar_file"
              onClick={handleSendSampleFile}
              title="Send File (Lightweight Cache)"
              className="p-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 hover:text-emerald-500 transition-colors"
            >
              <Paperclip className="w-4 h-4" />
            </button>

            {/* Code Snippet Modal */}
            <button
              id="btn_toolbar_code"
              onClick={() => setShowCodeModal(true)}
              title="Send Rust / Code Snippet"
              className="p-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 hover:text-emerald-500 transition-colors"
            >
              <Code className="w-4 h-4" />
            </button>

            {/* Voice Memo Simulator */}
            <button
              id="btn_toolbar_voice"
              onClick={() => setIsRecordingVoice(!isRecordingVoice)}
              title={isRecordingVoice ? 'Recording... click to stop' : 'Record Voice Note'}
              className={`p-1.5 rounded-lg transition-colors ${
                isRecordingVoice
                  ? 'bg-rose-500 text-white animate-pulse'
                  : 'hover:bg-black/5 dark:hover:bg-white/10 hover:text-emerald-500'
              }`}
            >
              <Mic className="w-4 h-4" />
            </button>

            {isRecordingVoice && (
              <span className="text-xs font-mono text-rose-500 font-bold ml-1 animate-pulse">
                Recording {recordingSeconds}s...
              </span>
            )}
          </div>

          <div className="flex items-center space-x-2 text-[11px] text-zinc-400 font-mono">
            <span>Enter to send, Shift+Enter for new line</span>
          </div>
        </div>

        {/* Textarea Input */}
        <div className="px-3 pb-2 flex items-end space-x-2">
          <textarea
            id="input_chat_message"
            ref={textareaRef}
            rows={2}
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a message or paste code..."
            className={`flex-1 resize-none bg-transparent border-none outline-none text-xs leading-relaxed p-1 placeholder-zinc-400 ${
              isDarkMode ? 'text-zinc-100' : 'text-zinc-900'
            }`}
          />

          <button
            id="btn_send_message"
            onClick={handleSend}
            disabled={!inputText.trim()}
            className={`px-4 py-1.5 rounded-lg font-medium text-xs flex items-center space-x-1.5 transition-all ${
              inputText.trim()
                ? 'bg-emerald-600 hover:bg-emerald-500 text-white shadow-sm cursor-pointer'
                : 'bg-zinc-200 dark:bg-zinc-800 text-zinc-400 cursor-not-allowed'
            }`}
          >
            <span>Send</span>
            <Send className="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* Code Snippet Modal */}
      {showCodeModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 z-50">
          <div className={`w-full max-w-lg rounded-2xl border shadow-2xl p-5 space-y-4 ${
            isDarkMode ? 'bg-[#1e1e24] border-zinc-800 text-zinc-100' : 'bg-white border-zinc-200 text-zinc-800'
          }`}>
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <Code className="w-5 h-5 text-emerald-500" />
                <h4 className="font-bold text-sm">Send Code Snippet</h4>
              </div>
              <button onClick={() => setShowCodeModal(false)} className="text-zinc-400 hover:text-zinc-600">
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex items-center space-x-2 text-xs">
              <label className="text-zinc-400 font-medium">Language:</label>
              <select
                value={codeLang}
                onChange={(e) => setCodeLang(e.target.value)}
                className={`px-2 py-1 rounded border text-xs outline-none ${
                  isDarkMode ? 'bg-[#18181c] border-zinc-700 text-zinc-200' : 'bg-zinc-100 border-zinc-300'
                }`}
              >
                <option value="rust">Rust</option>
                <option value="typescript">TypeScript</option>
                <option value="json">JSON</option>
                <option value="sql">SQL / SQLite</option>
                <option value="protobuf">Protobuf</option>
              </select>
            </div>

            <textarea
              rows={8}
              value={codeContent}
              onChange={(e) => setCodeContent(e.target.value)}
              placeholder="// Paste your Rust / Fenestra code here..."
              className={`w-full font-mono text-xs p-3 rounded-xl border outline-none ${
                isDarkMode ? 'bg-[#18181c] border-zinc-800 text-emerald-300' : 'bg-zinc-50 border-zinc-300 text-zinc-900'
              }`}
            />

            <div className="flex justify-end space-x-2 text-xs">
              <button
                onClick={() => setShowCodeModal(false)}
                className="px-3 py-1.5 rounded-lg border border-zinc-300 dark:border-zinc-700 hover:bg-black/5"
              >
                Cancel
              </button>
              <button
                onClick={handleSendCodeSnippet}
                disabled={!codeContent.trim()}
                className="px-4 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-medium disabled:opacity-50"
              >
                Send Code
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Conversation Details & Storage Drawer */}
      {showDetailsDrawer && (
        <div className={`w-64 border-l p-4 space-y-4 absolute right-0 top-14 bottom-0 shadow-2xl z-10 select-none overflow-y-auto ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800 text-zinc-200' : 'bg-white border-zinc-200 text-zinc-800'
        }`}>
          <div className="flex items-center justify-between border-b pb-2">
            <h4 className="text-xs font-bold">Chat Details & Storage</h4>
            <button onClick={() => setShowDetailsDrawer(false)} className="text-zinc-400 hover:text-zinc-600">
              <X className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="text-center space-y-2 py-2">
            <img src={conversation.avatar} alt={conversation.name} className="w-14 h-14 rounded-xl mx-auto object-cover shadow" />
            <h5 className="font-bold text-xs">{conversation.name}</h5>
            <p className="text-[10px] text-zinc-400 font-mono">Seq ID: #{conversation.lastSyncSeq}</p>
          </div>

          <div className="space-y-2 border-t pt-3 text-xs">
            <div className="flex justify-between text-zinc-500 dark:text-zinc-400 text-[11px]">
              <span>Cached Messages:</span>
              <strong className="text-zinc-700 dark:text-zinc-200 font-mono">{messages.length}</strong>
            </div>
            <div className="flex justify-between text-zinc-500 dark:text-zinc-400 text-[11px]">
              <span>LZ4 Storage:</span>
              <strong className="text-emerald-600 dark:text-emerald-400 font-mono">
                ~{Math.round(messages.reduce((acc, m) => acc + (m.compressedBytes || 60), 0) / 1024 * 10) / 10} KB
              </strong>
            </div>
            <div className="flex justify-between text-zinc-500 dark:text-zinc-400 text-[11px]">
              <span>Sync State:</span>
              <strong className="text-emerald-500 font-mono">Consistent</strong>
            </div>
          </div>

          <div className="border-t pt-3 space-y-2">
            <button
              onClick={() => onClearMessages(conversation.id)}
              className="w-full py-1.5 px-3 rounded-lg text-rose-500 hover:bg-rose-500/10 border border-rose-500/20 text-xs font-medium flex items-center justify-center space-x-1.5 transition-colors"
            >
              <Trash2 className="w-3.5 h-3.5" />
              <span>Clear Chat History</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
