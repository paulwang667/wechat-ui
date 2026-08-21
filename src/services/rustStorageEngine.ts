import { Message, StorageMetrics } from '../types/wechat';

class RustStorageEngineService {
  private messages: Map<string, Message> = new Map();
  private conversationIndex: Map<string, string[]> = new Map(); // convId -> messageId[]
  private searchInvertedIndex: Map<string, Set<string>> = new Map(); // keyword -> messageIds
  private blobRegistry: Map<string, { hash: string; size: number; refCount: number }> = new Map();
  
  // Storage size tracking
  private totalRawBytes: number = 0;
  private totalCompressedBytes: number = 0;
  private totalIndexBytes: number = 0;
  private totalWalBytes: number = 42 * 1024; // baseline WAL size (42KB)
  private cacheHits: number = 142;
  private cacheMisses: number = 3;
  private lastCompactionTime: number = Date.now() - 1000 * 3600 * 4;

  constructor() {
    this.loadFromLocalStorage();
  }

  private calculateMessageBytes(msg: Message): { raw: number; compressed: number } {
    const raw = JSON.stringify(msg).length * 2; // ~2 bytes per char
    // LZ4 / Snappy compression simulation: text compresses 60-75%, repetitive payloads up to 80%
    const ratio = msg.type === 'code' ? 0.32 : msg.type === 'text' ? 0.38 : 0.45;
    const compressed = Math.max(24, Math.floor(raw * ratio));
    return { raw, compressed };
  }

  public init(initialMessages: Message[]) {
    if (this.messages.size === 0) {
      initialMessages.forEach(msg => this.insertMessage(msg, false));
      this.persistToLocalStorage();
    }
  }

  public insertMessage(msg: Message, persist: boolean = true): Message {
    const { raw, compressed } = this.calculateMessageBytes(msg);
    const enrichedMsg: Message = {
      ...msg,
      rawBytes: raw,
      compressedBytes: compressed,
      isCompressed: true,
    };

    this.messages.set(msg.id, enrichedMsg);

    // Update conversation index
    const convList = this.conversationIndex.get(msg.conversationId) || [];
    if (!convList.includes(msg.id)) {
      convList.push(msg.id);
      this.conversationIndex.set(msg.conversationId, convList);
    }

    // Update FTS search inverted index
    this.indexMessageContent(msg.id, msg.content + ' ' + (msg.codeSnippet || '') + ' ' + (msg.fileName || ''));

    // Update blob deduplication if attachment
    if (msg.mediaUrl || msg.fileName) {
      const blobKey = msg.mediaUrl || msg.fileName || '';
      const existing = this.blobRegistry.get(blobKey);
      if (existing) {
        existing.refCount++;
      } else {
        this.blobRegistry.set(blobKey, {
          hash: 'sha256_' + Math.abs(blobKey.split('').reduce((a, b) => ((a << 5) - a) + b.charCodeAt(0), 0)).toString(16),
          size: enrichedMsg.rawBytes || 1024,
          refCount: 1,
        });
      }
    }

    // Storage stats
    this.totalRawBytes += raw;
    this.totalCompressedBytes += compressed;
    this.totalIndexBytes += 32; // index entry cost
    this.totalWalBytes += 48; // append-only WAL cost

    if (persist) {
      this.persistToLocalStorage();
    }

    return enrichedMsg;
  }

  private indexMessageContent(msgId: string, text: string) {
    if (!text) return;
    const tokens = text
      .toLowerCase()
      .replace(/[^\w\u4e00-\u9fa5]/g, ' ')
      .split(/\s+/)
      .filter(t => t.length > 0);

    for (const token of tokens) {
      if (!this.searchInvertedIndex.has(token)) {
        this.searchInvertedIndex.set(token, new Set());
      }
      this.searchInvertedIndex.get(token)!.add(msgId);
    }
  }

  public getMessagesByConversation(conversationId: string): Message[] {
    this.cacheHits++;
    const msgIds = this.conversationIndex.get(conversationId) || [];
    return msgIds
      .map(id => this.messages.get(id))
      .filter((m): m is Message => m !== undefined)
      .sort((a, b) => a.timestamp - b.timestamp);
  }

  public searchMessages(query: string, conversationId?: string): { messages: Message[]; searchTimeMs: number } {
    const startTime = performance.now();
    const cleanQuery = query.toLowerCase().trim();
    if (!cleanQuery) return { messages: [], searchTimeMs: 0 };

    const results: Message[] = [];
    const tokens = cleanQuery.split(/\s+/).filter(t => t.length > 0);

    // Fast Inverted Index Lookup
    const candidateIds = new Set<string>();
    tokens.forEach((token, idx) => {
      // Find matching keys in inverted index
      for (const [key, idSet] of this.searchInvertedIndex.entries()) {
        if (key.includes(token)) {
          idSet.forEach(id => candidateIds.add(id));
        }
      }
    });

    candidateIds.forEach(id => {
      const msg = this.messages.get(id);
      if (msg) {
        if (conversationId && msg.conversationId !== conversationId) return;
        results.push(msg);
      }
    });

    // Fallback direct scan if query is tiny or single char
    if (results.length === 0) {
      for (const msg of this.messages.values()) {
        if (conversationId && msg.conversationId !== conversationId) continue;
        if (msg.content.toLowerCase().includes(cleanQuery) || 
            (msg.fileName && msg.fileName.toLowerCase().includes(cleanQuery)) ||
            (msg.codeSnippet && msg.codeSnippet.toLowerCase().includes(cleanQuery))) {
          results.push(msg);
        }
      }
    }

    results.sort((a, b) => b.timestamp - a.timestamp);
    const searchTimeMs = Number((performance.now() - startTime).toFixed(2));
    return { messages: results, searchTimeMs };
  }

  public updateMessageStatus(id: string, status: Message['status'], seqId?: number) {
    const msg = this.messages.get(id);
    if (msg) {
      msg.status = status;
      if (seqId) msg.seqId = seqId;
      this.messages.set(id, { ...msg });
      this.persistToLocalStorage();
    }
  }

  public clearConversation(convId: string) {
    const msgIds = this.conversationIndex.get(convId) || [];
    msgIds.forEach(id => {
      const msg = this.messages.get(id);
      if (msg) {
        this.totalRawBytes = Math.max(0, this.totalRawBytes - (msg.rawBytes || 0));
        this.totalCompressedBytes = Math.max(0, this.totalCompressedBytes - (msg.compressedBytes || 0));
        this.messages.delete(id);
      }
    });
    this.conversationIndex.delete(convId);
    this.persistToLocalStorage();
  }

  public batchInsertStressTest(count: number, convId: string): { durationMs: number; inserted: number } {
    const startTime = performance.now();
    const sampleSentences = [
      "Rust memory safety without garbage collection yields predictable tail latency.",
      "Fenestra rendering pipeline achieves 120 FPS by compiling UI layout trees directly to GPU vertices.",
      "Incremental sync sequence confirmed. Vector clock synchronized across peer nodes.",
      "Zero-copy serialization with rkyv reduces deserialization overhead to exactly 0ns.",
      "SQLite WAL auto-checkpoint pruned 14MB of fragmented pages.",
      "Benchmarking message sync throughput: 45,000 msgs/sec on single core Tokio worker."
    ];

    for (let i = 0; i < count; i++) {
      const text = sampleSentences[i % sampleSentences.length] + ` [Stress Test Seq #${i + 1}]`;
      const msg: Message = {
        id: `stress_${Date.now()}_${i}`,
        conversationId: convId,
        senderId: i % 2 === 0 ? 'bot_rust_engine' : 'user_self',
        senderName: i % 2 === 0 ? 'Rust Sync Engine (Bot)' : 'Rust Dev (You)',
        senderAvatar: i % 2 === 0 
          ? 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=150&auto=format&fit=crop&q=80'
          : 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=150&auto=format&fit=crop&q=80',
        isSelf: i % 2 !== 0,
        type: 'text',
        content: text,
        timestamp: Date.now() - (count - i) * 1000,
        seqId: 2000 + i,
        status: 'synced',
      };
      this.insertMessage(msg, false);
    }

    this.persistToLocalStorage();
    const durationMs = Number((performance.now() - startTime).toFixed(2));
    return { durationMs, inserted: count };
  }

  public vacuumAndCompact(): { freedBytes: number; newTotalBytes: number; durationMs: number } {
    const startTime = performance.now();
    const beforeBytes = this.totalCompressedBytes + this.totalWalBytes + this.totalIndexBytes;
    
    // Prune WAL into primary store
    const freedWal = Math.floor(this.totalWalBytes * 0.75);
    this.totalWalBytes = Math.max(8192, this.totalWalBytes - freedWal);
    
    // Defragment B-tree index
    const freedIndex = Math.floor(this.totalIndexBytes * 0.2);
    this.totalIndexBytes = Math.max(1024, this.totalIndexBytes - freedIndex);
    
    const freedBytes = freedWal + freedIndex;
    const newTotalBytes = this.totalCompressedBytes + this.totalWalBytes + this.totalIndexBytes;
    this.lastCompactionTime = Date.now();
    
    const durationMs = Number((performance.now() - startTime).toFixed(2));
    return { freedBytes, newTotalBytes, durationMs };
  }

  public getMetrics(): StorageMetrics {
    const totalMsgs = this.messages.size;
    const totalConvs = this.conversationIndex.size;
    const raw = this.totalRawBytes || 24000;
    const comp = this.totalCompressedBytes || 8200;
    const saved = Math.max(0, raw - comp);
    const ratio = raw > 0 ? Number(((1 - comp / raw) * 100).toFixed(1)) : 68.4;
    
    // Blob deduplication stats
    let blobSaved = 0;
    let blobDedupCount = 0;
    this.blobRegistry.forEach(b => {
      if (b.refCount > 1) {
        blobDedupCount += (b.refCount - 1);
        blobSaved += (b.refCount - 1) * b.size;
      }
    });

    const totalCache = this.cacheHits + this.cacheMisses;
    const cacheHitRatio = totalCache > 0 ? Number(((this.cacheHits / totalCache) * 100).toFixed(1)) : 98.2;

    return {
      totalMessages: totalMsgs,
      totalConversations: totalConvs,
      rawStorageBytes: raw,
      compressedStorageBytes: comp,
      savedStorageBytes: saved,
      compressionRatio: ratio,
      indexSizeBytes: this.totalIndexBytes || 4096,
      walLogSizeBytes: this.totalWalBytes,
      blobDeduplicatedCount: blobDedupCount,
      blobSavedBytes: blobSaved,
      activeMemoryCacheBytes: Math.floor(comp * 0.4), // 40% hot in memory ringbuffer
      cacheHitRatio,
      lastCompactionTime: this.lastCompactionTime,
    };
  }

  public exportDatabaseJson(): string {
    const all = Array.from(this.messages.values());
    return JSON.stringify({
      schema_version: 'fenestra_wechat_v1_rust',
      exported_at: new Date().toISOString(),
      messages_count: all.length,
      metrics: this.getMetrics(),
      messages: all,
    }, null, 2);
  }

  public importDatabaseJson(jsonData: string): { success: boolean; importedCount: number; error?: string } {
    try {
      const parsed = JSON.parse(jsonData);
      if (!parsed.messages || !Array.isArray(parsed.messages)) {
        return { success: false, importedCount: 0, error: 'Invalid database payload: missing messages array' };
      }
      
      let count = 0;
      parsed.messages.forEach((m: Message) => {
        if (m.id && m.conversationId && m.content !== undefined) {
          this.insertMessage(m, false);
          count++;
        }
      });
      this.persistToLocalStorage();
      return { success: true, importedCount: count };
    } catch (e: any) {
      return { success: false, importedCount: 0, error: e.message || 'Failed to parse JSON' };
    }
  }

  private persistToLocalStorage() {
    try {
      const raw = Array.from(this.messages.values()).slice(-200); // keep recent 200 in localStorage
      localStorage.setItem('rust_wechat_messages_cache', JSON.stringify(raw));
    } catch (e) {
      // ignore storage quota in browser demo
    }
  }

  private loadFromLocalStorage() {
    try {
      const cached = localStorage.getItem('rust_wechat_messages_cache');
      if (cached) {
        const msgs: Message[] = JSON.parse(cached);
        msgs.forEach(m => this.insertMessage(m, false));
      }
    } catch (e) {
      // ignore
    }
  }
}

export const rustStorageEngine = new RustStorageEngineService();
