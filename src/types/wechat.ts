export type MessageType = 'text' | 'image' | 'voice' | 'file' | 'code' | 'system' | 'quote';

export type SyncStatus = 'pending' | 'sending' | 'synced' | 'failed';

export interface Message {
  id: string;
  conversationId: string;
  senderId: string;
  senderName: string;
  senderAvatar: string;
  isSelf: boolean;
  type: MessageType;
  content: string;
  timestamp: number;
  seqId: number; // Incremental sequence ID for sync
  status: SyncStatus;
  
  // Optional rich content
  mediaUrl?: string;
  fileName?: string;
  fileSize?: string;
  voiceDuration?: number; // seconds
  codeLanguage?: string;
  codeSnippet?: string;
  quotedMessage?: {
    senderName: string;
    content: string;
  };
  
  // Storage & Compression metadata
  rawBytes?: number;
  compressedBytes?: number;
  isCompressed?: boolean;
}

export interface Conversation {
  id: string;
  name: string;
  avatar: string;
  isGroup: boolean;
  groupMembers?: Contact[];
  lastMessage?: string;
  lastMessageTime: number;
  unreadCount: number;
  isPinned: boolean;
  isMuted: boolean;
  draft?: string;
  lastSyncSeq: number;
}

export interface Contact {
  id: string;
  name: string;
  alias?: string;
  avatar: string;
  wechatId: string;
  region: string;
  bio: string;
  category: 'friend' | 'colleague' | 'group' | 'bot' | 'service';
  onlineStatus: 'online' | 'busy' | 'offline';
}

export interface SyncLogEntry {
  id: string;
  timestamp: number;
  type: 'PULL_REQ' | 'PULL_RESP' | 'PUSH_MSG' | 'ACK' | 'CONFLICT_RESOLVE' | 'COMPACT' | 'HEARTBEAT';
  seq: number;
  details: string;
  latencyMs: number;
  bytesTransferred: number;
  success: boolean;
}

export interface StorageMetrics {
  totalMessages: number;
  totalConversations: number;
  rawStorageBytes: number;
  compressedStorageBytes: number;
  savedStorageBytes: number;
  compressionRatio: number; // e.g. 68%
  indexSizeBytes: number;
  walLogSizeBytes: number;
  blobDeduplicatedCount: number;
  blobSavedBytes: number;
  activeMemoryCacheBytes: number;
  cacheHitRatio: number; // e.g. 98.4%
  lastCompactionTime: number;
}

export interface NetworkCondition {
  mode: 'optimal' | 'wifi' | 'cellular_4g' | 'high_latency' | 'packet_loss' | 'offline';
  latencyMs: number;
  packetLossRate: number; // 0.0 to 1.0
  bandwidthKbps: number;
}

export interface PerformanceBenchmarkResult {
  messageCount: number;
  insertTimeMs: number;
  insertRateMsgPerSec: number;
  ftsSearchTimeMs: number;
  scrollFps: number;
  memoryUsageMb: number;
  fenestraRenderTimeMs: number;
}
