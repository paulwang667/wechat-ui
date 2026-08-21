import { Message, NetworkCondition, SyncLogEntry } from '../types/wechat';
import { rustStorageEngine } from './rustStorageEngine';

type SyncListener = (event: { type: string; payload: any }) => void;

class SyncEngineService {
  private currentSeq: number = 1042;
  private networkCondition: NetworkCondition = {
    mode: 'optimal',
    latencyMs: 12,
    packetLossRate: 0,
    bandwidthKbps: 50000,
  };
  
  private outboxQueue: Message[] = [];
  private syncLogs: SyncLogEntry[] = [];
  private listeners: Set<SyncListener> = new Set();
  private heartbeatTimer: any = null;
  private isSyncing: boolean = false;

  constructor() {
    this.startHeartbeat();
  }

  public setNetworkCondition(condition: Partial<NetworkCondition>) {
    this.networkCondition = { ...this.networkCondition, ...condition };
    this.emit('network_changed', this.networkCondition);
    this.addLog({
      type: 'HEARTBEAT',
      seq: this.currentSeq,
      details: `Network mode set to [${this.networkCondition.mode.toUpperCase()}]: Latency ${this.networkCondition.latencyMs}ms, Loss ${(this.networkCondition.packetLossRate * 100).toFixed(0)}%`,
      latencyMs: this.networkCondition.latencyMs,
      bytesTransferred: 64,
      success: true,
    });

    // If restored from offline, flush outbox
    if (this.networkCondition.mode !== 'offline' && this.outboxQueue.length > 0) {
      this.flushOutbox();
    }
  }

  public getNetworkCondition(): NetworkCondition {
    return this.networkCondition;
  }

  public getCurrentSeq(): number {
    return this.currentSeq;
  }

  public getSyncLogs(): SyncLogEntry[] {
    return [...this.syncLogs];
  }

  public getOutbox(): Message[] {
    return [...this.outboxQueue];
  }

  public subscribe(listener: SyncListener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private emit(type: string, payload: any) {
    this.listeners.forEach(l => l({ type, payload }));
  }

  private addLog(entry: Omit<SyncLogEntry, 'id' | 'timestamp'>) {
    const log: SyncLogEntry = {
      id: 'log_' + Date.now() + '_' + Math.random().toString(36).substr(2, 4),
      timestamp: Date.now(),
      ...entry,
    };
    this.syncLogs.unshift(log);
    if (this.syncLogs.length > 100) {
      this.syncLogs.pop();
    }
    this.emit('log_added', log);
  }

  public async sendMessage(msg: Omit<Message, 'id' | 'seqId' | 'status'> & { id?: string }): Promise<Message> {
    const isOffline = this.networkCondition.mode === 'offline';
    const isPacketLost = Math.random() < this.networkCondition.packetLossRate;
    
    // Optimistic local store
    const nextSeq = ++this.currentSeq;
    const initialStatus = isOffline ? 'pending' : 'sending';

    const fullMsg: Message = {
      id: msg.id || 'msg_out_' + Date.now() + '_' + Math.random().toString(36).substr(2, 6),
      ...msg,
      seqId: nextSeq,
      status: initialStatus,
    };

    // Store in Rust local engine immediately
    const stored = rustStorageEngine.insertMessage(fullMsg);
    this.emit('message_saved', stored);

    if (isOffline) {
      this.outboxQueue.push(stored);
      this.addLog({
        type: 'PUSH_MSG',
        seq: nextSeq,
        details: `[OFFLINE] Message queued in Outbox (Seq #${nextSeq})`,
        latencyMs: 0,
        bytesTransferred: stored.compressedBytes || 64,
        success: false,
      });
      return stored;
    }

    // Network delivery simulation
    this.addLog({
      type: 'PUSH_MSG',
      seq: nextSeq,
      details: `Sending protobuf delta payload to Sync Gate (Seq #${nextSeq})`,
      latencyMs: this.networkCondition.latencyMs,
      bytesTransferred: stored.compressedBytes || 64,
      success: !isPacketLost,
    });

    setTimeout(() => {
      if (isPacketLost) {
        // Packet loss: mark failed and put to outbox for retry
        rustStorageEngine.updateMessageStatus(stored.id, 'failed');
        this.outboxQueue.push({ ...stored, status: 'failed' });
        this.emit('message_status_changed', { id: stored.id, status: 'failed' });
        this.addLog({
          type: 'PUSH_MSG',
          seq: nextSeq,
          details: `Packet drop simulated! Retrying via Tokio async exponential backoff...`,
          latencyMs: this.networkCondition.latencyMs,
          bytesTransferred: 0,
          success: false,
        });

        // Auto retry after 2.5s
        setTimeout(() => this.flushOutbox(), 2500);
      } else {
        // Successful ACK
        rustStorageEngine.updateMessageStatus(stored.id, 'synced');
        this.emit('message_status_changed', { id: stored.id, status: 'synced' });
        this.addLog({
          type: 'ACK',
          seq: nextSeq,
          details: `Server ACK received for Seq #${nextSeq}. Delta confirmed across cluster.`,
          latencyMs: Math.floor(this.networkCondition.latencyMs * 1.2),
          bytesTransferred: 32,
          success: true,
        });

        // If message is sent to a bot or peer contact, trigger simulated intelligent response
        this.triggerSimulatedResponse(stored);
      }
    }, Math.max(20, this.networkCondition.latencyMs));

    return stored;
  }

  public async flushOutbox() {
    if (this.networkCondition.mode === 'offline' || this.outboxQueue.length === 0) return;
    
    const pending = [...this.outboxQueue];
    this.outboxQueue = [];

    this.addLog({
      type: 'PULL_REQ',
      seq: this.currentSeq,
      details: `Flushing ${pending.length} outbox message(s)...`,
      latencyMs: this.networkCondition.latencyMs,
      bytesTransferred: pending.length * 96,
      success: true,
    });

    for (const msg of pending) {
      rustStorageEngine.updateMessageStatus(msg.id, 'synced');
      this.emit('message_status_changed', { id: msg.id, status: 'synced' });
      this.addLog({
        type: 'ACK',
        seq: msg.seqId,
        details: `Outbox message Seq #${msg.seqId} synced successfully.`,
        latencyMs: this.networkCondition.latencyMs,
        bytesTransferred: 32,
        success: true,
      });
      this.triggerSimulatedResponse(msg);
    }
  }

  private triggerSimulatedResponse(userMsg: Message) {
    // Only reply if sending to non-self conversations
    if (userMsg.conversationId === 'conv_rust_bot') {
      setTimeout(() => {
        const replies = [
          `⚡ [Rust Engine ACK] Received: "${userMsg.content}". In-memory ring buffer synchronized in 0.04ms.`,
          `📦 Stored with LZ4. Compression ratio is currently 68.2%. Zero-copy buffer pointer updated.`,
          `🚀 Sync sequence monotonic increment confirmed at Seq #${this.currentSeq + 1}.`,
          `💡 Tip: You can switch network condition to "Packet Loss" or "High Latency" in the Diagnostics tab to test outbox auto-recovery!`,
        ];
        const randomReply = replies[Math.floor(Math.random() * replies.length)];
        this.receiveIncomingMessage({
          conversationId: userMsg.conversationId,
          senderId: 'bot_rust_engine',
          senderName: 'Rust Sync Engine (Bot)',
          senderAvatar: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=150&auto=format&fit=crop&q=80',
          isSelf: false,
          type: 'text',
          content: randomReply,
        });
      }, 700 + Math.random() * 600);
    } else if (userMsg.conversationId === 'conv_fenestra_core') {
      setTimeout(() => {
        this.receiveIncomingMessage({
          conversationId: userMsg.conversationId,
          senderId: 'contact_fenestra_core',
          senderName: 'Fenestra Lead Dev',
          senderAvatar: 'https://images.unsplash.com/photo-1633356122544-f134324a6cee?w=150&auto=format&fit=crop&q=80',
          isSelf: false,
          type: 'text',
          content: `Great observation on "${userMsg.content}". Fenestra's layout engine uses direct SIMD math for 120 FPS frame pacing!`,
        });
      }, 1200);
    } else if (userMsg.conversationId === 'conv_sarah') {
      setTimeout(() => {
        this.receiveIncomingMessage({
          conversationId: userMsg.conversationId,
          senderId: 'contact_sarah',
          senderName: 'Sarah Chen (Tech Lead)',
          senderAvatar: 'https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=150&auto=format&fit=crop&q=80',
          isSelf: false,
          type: 'text',
          content: `Checked the sync metrics! Delta sync overhead is under 40 bytes per text frame. Solid progress.`,
        });
      }, 1500);
    }
  }

  public receiveIncomingMessage(msg: Omit<Message, 'id' | 'timestamp' | 'seqId' | 'status'>) {
    const nextSeq = ++this.currentSeq;
    const incoming: Message = {
      ...msg,
      id: 'msg_in_' + Date.now() + '_' + Math.random().toString(36).substr(2, 4),
      timestamp: Date.now(),
      seqId: nextSeq,
      status: 'synced',
    };

    const stored = rustStorageEngine.insertMessage(incoming);
    this.addLog({
      type: 'PULL_RESP',
      seq: nextSeq,
      details: `Incremental Delta push arrived: "${incoming.content.slice(0, 32)}..." from ${incoming.senderName}`,
      latencyMs: this.networkCondition.latencyMs,
      bytesTransferred: stored.compressedBytes || 80,
      success: true,
    });

    this.emit('incoming_message', stored);
    return stored;
  }

  private startHeartbeat() {
    this.heartbeatTimer = setInterval(() => {
      if (this.networkCondition.mode === 'offline') return;
      this.addLog({
        type: 'HEARTBEAT',
        seq: this.currentSeq,
        details: `Sync Keepalive ping • Client Seq #${this.currentSeq} in sync with remote gateway`,
        latencyMs: this.networkCondition.latencyMs,
        bytesTransferred: 16,
        success: true,
      });
    }, 15000);
  }

  public triggerManualSync() {
    this.isSyncing = true;
    this.addLog({
      type: 'PULL_REQ',
      seq: this.currentSeq,
      details: `Manual sync pull requested (SyncKey: ${this.currentSeq})`,
      latencyMs: this.networkCondition.latencyMs,
      bytesTransferred: 64,
      success: true,
    });

    setTimeout(() => {
      this.isSyncing = false;
      this.addLog({
        type: 'PULL_RESP',
        seq: this.currentSeq,
        details: `Sync response: 0 delta conflicts. Local storage up to date.`,
        latencyMs: this.networkCondition.latencyMs,
        bytesTransferred: 32,
        success: true,
      });
      this.emit('sync_completed', { seq: this.currentSeq });
    }, Math.max(50, this.networkCondition.latencyMs * 2));
  }
}

export const syncEngine = new SyncEngineService();
