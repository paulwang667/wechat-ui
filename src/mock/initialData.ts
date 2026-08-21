import { Contact, Conversation, Message } from '../types/wechat';

export const CURRENT_USER: Contact = {
  id: 'user_self',
  name: 'Rust Dev (You)',
  alias: 'Ferris',
  avatar: 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=150&auto=format&fit=crop&q=80',
  wechatId: 'rust_rustacean_01',
  region: 'Hangzhou, China',
  bio: 'Systems Programmer • Rust + Fenestra enthusiast • Zero-cost abstractions',
  category: 'friend',
  onlineStatus: 'online',
};

export const INITIAL_CONTACTS: Contact[] = [
  {
    id: 'bot_rust_engine',
    name: 'Rust Sync Engine (Bot)',
    alias: 'Rust Core Bot',
    avatar: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=150&auto=format&fit=crop&q=80',
    wechatId: 'rust_engine_ai',
    region: 'Tokyo, Japan',
    bio: 'Simulated high-concurrency Rust sync node (Seq ID engine, Zero-Copy RingBuffer)',
    category: 'bot',
    onlineStatus: 'online',
  },
  {
    id: 'contact_fenestra_core',
    name: 'Fenestra UI Framework',
    alias: 'Fenestra Architecture Guild',
    avatar: 'https://images.unsplash.com/photo-1633356122544-f134324a6cee?w=150&auto=format&fit=crop&q=80',
    wechatId: 'fenestra_org',
    region: 'Stockholm, Sweden',
    bio: 'Ultra-lightweight native desktop GUI framework in Rust. 15MB RAM, 120 FPS.',
    category: 'group',
    onlineStatus: 'online',
  },
  {
    id: 'contact_linus',
    name: 'Linus Torvalds',
    alias: 'Linus',
    avatar: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=150&auto=format&fit=crop&q=80',
    wechatId: 'torvalds_kernel',
    region: 'Portland, USA',
    bio: 'Talk is cheap. Show me the code.',
    category: 'friend',
    onlineStatus: 'busy',
  },
  {
    id: 'contact_sarah',
    name: 'Sarah Chen (Tech Lead)',
    alias: 'Sarah',
    avatar: 'https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=150&auto=format&fit=crop&q=80',
    wechatId: 'sarah_arch',
    region: 'Shenzhen, China',
    bio: 'Optimizing client storage & sync latency. Rust in production.',
    category: 'colleague',
    onlineStatus: 'online',
  },
  {
    id: 'contact_wechat_team',
    name: 'WeChat Desktop Team',
    alias: 'WeChat Official',
    avatar: 'https://images.unsplash.com/photo-1550751827-4bd374c3f58b?w=150&auto=format&fit=crop&q=80',
    wechatId: 'wechat_desktop_official',
    region: 'Guangzhou, China',
    bio: 'Official WeChat Desktop announcements & performance optimization changelog',
    category: 'service',
    onlineStatus: 'offline',
  },
  {
    id: 'contact_rust_china',
    name: 'Rust Systems Dev Community',
    alias: 'Rust Desktop SIG',
    avatar: 'https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?w=150&auto=format&fit=crop&q=80',
    wechatId: 'rust_china_sig',
    region: 'Beijing, China',
    bio: 'Rust GUI, cross-platform compilation, sled, redb, and tokio microservices',
    category: 'group',
    onlineStatus: 'online',
  }
];

export const INITIAL_CONVERSATIONS: Conversation[] = [
  {
    id: 'conv_rust_bot',
    name: 'Rust Sync Engine (Bot)',
    avatar: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=150&auto=format&fit=crop&q=80',
    isGroup: false,
    lastMessage: 'Sync Engine ready. Sequence ID: 1042. Local storage compressed with LZ4 (ratio 68.4%).',
    lastMessageTime: Date.now() - 1000 * 60 * 2,
    unreadCount: 0,
    isPinned: true,
    isMuted: false,
    lastSyncSeq: 1042,
  },
  {
    id: 'conv_fenestra_core',
    name: 'Fenestra UI Framework',
    avatar: 'https://images.unsplash.com/photo-1633356122544-f134324a6cee?w=150&auto=format&fit=crop&q=80',
    isGroup: true,
    lastMessage: 'fenestra-v0.9.4 released: zero-allocation virtualized list rendering ready!',
    lastMessageTime: Date.now() - 1000 * 60 * 15,
    unreadCount: 2,
    isPinned: true,
    isMuted: false,
    lastSyncSeq: 890,
  },
  {
    id: 'conv_sarah',
    name: 'Sarah Chen (Tech Lead)',
    avatar: 'https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=150&auto=format&fit=crop&q=80',
    isGroup: false,
    lastMessage: 'Can you benchmark the SQLite WAL compaction vs our Rust redb store?',
    lastMessageTime: Date.now() - 1000 * 60 * 45,
    unreadCount: 0,
    isPinned: false,
    isMuted: false,
    lastSyncSeq: 560,
  },
  {
    id: 'conv_linus',
    name: 'Linus Torvalds',
    avatar: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=150&auto=format&fit=crop&q=80',
    isGroup: false,
    lastMessage: 'Good to see you ditching heavy Electron. 15MB RAM baseline is the way.',
    lastMessageTime: Date.now() - 1000 * 3600 * 2,
    unreadCount: 0,
    isPinned: false,
    isMuted: false,
    lastSyncSeq: 310,
  },
  {
    id: 'conv_rust_china',
    name: 'Rust Systems Dev Community',
    avatar: 'https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?w=150&auto=format&fit=crop&q=80',
    isGroup: true,
    lastMessage: 'WeChat client with Rust + Fenestra is lightning fast. Zero frame drops!',
    lastMessageTime: Date.now() - 1000 * 3600 * 5,
    unreadCount: 0,
    isPinned: false,
    isMuted: true,
    lastSyncSeq: 204,
  },
  {
    id: 'conv_wechat_team',
    name: 'WeChat Desktop Team',
    avatar: 'https://images.unsplash.com/photo-1550751827-4bd374c3f58b?w=150&auto=format&fit=crop&q=80',
    isGroup: false,
    lastMessage: 'Notice: Sync protocol v3 with protobuf delta compaction is enabled.',
    lastMessageTime: Date.now() - 1000 * 3600 * 24,
    unreadCount: 0,
    isPinned: false,
    isMuted: false,
    lastSyncSeq: 98,
  }
];

export const INITIAL_MESSAGES: Message[] = [
  // Conversation with Bot
  {
    id: 'msg_bot_01',
    conversationId: 'conv_rust_bot',
    senderId: 'bot_rust_engine',
    senderName: 'Rust Sync Engine (Bot)',
    senderAvatar: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'system',
    content: '⚡ [Rust Native Core] Initialized storage engine (SQLite/redb + LZ4 compression + FTS5 full text index). Base memory: 14.8MB.',
    timestamp: Date.now() - 1000 * 60 * 20,
    seqId: 1039,
    status: 'synced',
    rawBytes: 256,
    compressedBytes: 84,
  },
  {
    id: 'msg_bot_02',
    conversationId: 'conv_rust_bot',
    senderId: 'user_self',
    senderName: 'Rust Dev (You)',
    senderAvatar: 'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=150&auto=format&fit=crop&q=80',
    isSelf: true,
    type: 'text',
    content: 'Hello! I am testing the new incremental message sync protocol with zero-copy deserialization.',
    timestamp: Date.now() - 1000 * 60 * 18,
    seqId: 1040,
    status: 'synced',
    rawBytes: 96,
    compressedBytes: 38,
  },
  {
    id: 'msg_bot_03',
    conversationId: 'conv_rust_bot',
    senderId: 'bot_rust_engine',
    senderName: 'Rust Sync Engine (Bot)',
    senderAvatar: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'code',
    content: 'Here is our core Rust Tokio sync worker loop:',
    codeLanguage: 'rust',
    codeSnippet: `// Rust Tokio Async Incremental Sync Loop
pub async fn run_sync_worker(
    mut rx: mpsc::Receiver<OutboxMsg>,
    db: Arc<StorageEngine>,
    sync_key: Arc<AtomicU64>,
) -> Result<()> {
    while let Some(msg) = rx.recv().await {
        let compressed = lz4_flex::compress_prepend_size(&msg.payload);
        let seq = sync_key.fetch_add(1, Ordering::SeqCst);
        db.insert_message(seq, &compressed)?;
        send_ack_to_server(seq).await?;
    }
    Ok(())
}`,
    timestamp: Date.now() - 1000 * 60 * 10,
    seqId: 1041,
    status: 'synced',
    rawBytes: 480,
    compressedBytes: 152,
  },
  {
    id: 'msg_bot_04',
    conversationId: 'conv_rust_bot',
    senderId: 'bot_rust_engine',
    senderName: 'Rust Sync Engine (Bot)',
    senderAvatar: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'text',
    content: 'Sync Engine ready. Sequence ID: 1042. Local storage compressed with LZ4 (ratio 68.4%). Send me any message to test real-time delta sync!',
    timestamp: Date.now() - 1000 * 60 * 2,
    seqId: 1042,
    status: 'synced',
    rawBytes: 140,
    compressedBytes: 46,
  },

  // Fenestra UI Framework
  {
    id: 'msg_fen_01',
    conversationId: 'conv_fenestra_core',
    senderId: 'contact_fenestra_core',
    senderName: 'Fenestra Lead Dev',
    senderAvatar: 'https://images.unsplash.com/photo-1633356122544-f134324a6cee?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'text',
    content: 'Welcome to Fenestra! We just achieved 120 FPS rendering on 4K monitors while keeping CPU usage under 0.8% during active scrolling.',
    timestamp: Date.now() - 1000 * 60 * 30,
    seqId: 888,
    status: 'synced',
  },
  {
    id: 'msg_fen_02',
    conversationId: 'conv_fenestra_core',
    senderId: 'contact_fenestra_core',
    senderName: 'Fenestra Lead Dev',
    senderAvatar: 'https://images.unsplash.com/photo-1633356122544-f134324a6cee?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'voice',
    content: '[Voice Memo: 6 seconds]',
    voiceDuration: 6,
    timestamp: Date.now() - 1000 * 60 * 20,
    seqId: 889,
    status: 'synced',
  },
  {
    id: 'msg_fen_03',
    conversationId: 'conv_fenestra_core',
    senderId: 'contact_fenestra_core',
    senderName: 'Fenestra Lead Dev',
    senderAvatar: 'https://images.unsplash.com/photo-1633356122544-f134324a6cee?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'file',
    content: 'fenestra_benchmark_report_v0.9.4.pdf',
    fileName: 'fenestra_benchmark_report_v0.9.4.pdf',
    fileSize: '1.4 MB',
    timestamp: Date.now() - 1000 * 60 * 15,
    seqId: 890,
    status: 'synced',
  },

  // Sarah Chen
  {
    id: 'msg_sarah_01',
    conversationId: 'conv_sarah',
    senderId: 'contact_sarah',
    senderName: 'Sarah Chen',
    senderAvatar: 'https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'text',
    content: 'Hey! The SQLite WAL auto-checkpointing helped reduce our disk write amplification by 74%.',
    timestamp: Date.now() - 1000 * 60 * 60,
    seqId: 559,
    status: 'synced',
  },
  {
    id: 'msg_sarah_02',
    conversationId: 'conv_sarah',
    senderId: 'contact_sarah',
    senderName: 'Sarah Chen',
    senderAvatar: 'https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'text',
    content: 'Can you benchmark the SQLite WAL compaction vs our Rust redb store?',
    timestamp: Date.now() - 1000 * 60 * 45,
    seqId: 560,
    status: 'synced',
  },

  // Linus
  {
    id: 'msg_linus_01',
    conversationId: 'conv_linus',
    senderId: 'contact_linus',
    senderName: 'Linus Torvalds',
    senderAvatar: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=150&auto=format&fit=crop&q=80',
    isSelf: false,
    type: 'text',
    content: 'Good to see you ditching heavy Electron. 15MB RAM baseline is the way. Keep the kernel calls clean.',
    timestamp: Date.now() - 1000 * 3600 * 2,
    seqId: 310,
    status: 'synced',
  }
];

export const RUST_ARCHITECTURE_SNIPPETS = [
  {
    filename: 'src/storage/engine.rs',
    title: 'Rust Lightweight Storage Engine (B-Tree + LZ4 + WAL)',
    description: 'Zero-copy in-memory buffer with SQLite WAL mode and column-oriented chunk compression.',
    code: `use std::sync::Arc;
use parking_lot::RwLock;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use rusqlite::{params, Connection, Result};

pub struct RustStorageEngine {
    db: Arc<RwLock<Connection>>,
    memory_ring_buffer: Arc<RwLock<RingBuffer<MessagePayload>>>,
}

impl RustStorageEngine {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        // High-performance Pragmas
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = -8000; -- 8MB cache
            PRAGMA mmap_size = 67108864; -- 64MB memory-mapped I/O
            
            CREATE TABLE IF NOT EXISTS messages (
                seq_id INTEGER PRIMARY KEY,
                conv_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                msg_type INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                compressed_payload BLOB NOT NULL,
                payload_crc32 INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_conv_time ON messages(conv_id, timestamp);
        ")?;
        
        Ok(Self {
            db: Arc::new(RwLock::new(conn)),
            memory_ring_buffer: Arc::new(RwLock::new(RingBuffer::with_capacity(1024))),
        })
    }

    /// Fast zero-copy store with LZ4 chunk compression
    pub fn store_message(&self, msg: &Message) -> Result<()> {
        let raw_bytes = serde_json::to_vec(msg).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let compressed = compress_prepend_size(&raw_bytes);
        let crc = crc32fast::hash(&compressed);

        let conn = self.db.write();
        conn.execute(
            "INSERT INTO messages (seq_id, conv_id, sender_id, msg_type, timestamp, compressed_payload, payload_crc32)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![msg.seq_id, msg.conversation_id, msg.sender_id, msg.msg_type as u8, msg.timestamp, compressed, crc],
        )?;

        // Update In-Memory Ring Buffer for immediate 0ms read
        self.memory_ring_buffer.write().push(msg.clone());
        Ok(())
    }

    /// Compact & Vacuum DB freeing up fragmented space
    pub fn vacuum_compact(&self) -> Result<CompactionReport> {
        let conn = self.db.write();
        let before_size = std::fs::metadata("wechat.db")?.len();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")?;
        let after_size = std::fs::metadata("wechat.db")?.len();
        
        Ok(CompactionReport {
            saved_bytes: before_size.saturating_sub(after_size),
            new_size: after_size,
        })
    }
}`
  },
  {
    filename: 'src/sync/protocol.rs',
    title: 'Incremental Sequence Sync Protocol (Protobuf / Delta Sync)',
    description: 'Guarantees message ordering, vector clock convergence, and zero lost packets over flaky networks.',
    code: `use prost::Message as ProstMessage;
use std::collections::VecDeque;

#[derive(Clone, PartialEq, ProstMessage)]
pub struct SyncRequest {
    #[prost(uint64, tag = "1")]
    pub client_sync_key: u64,
    #[prost(uint32, tag = "2")]
    pub batch_limit: u32,
    #[prost(string, tag = "3")]
    pub device_fingerprint: String,
}

#[derive(Clone, PartialEq, ProstMessage)]
pub struct SyncResponse {
    #[prost(uint64, tag = "1")]
    pub new_sync_key: u64,
    #[prost(bool, tag = "2")]
    pub has_more: bool,
    #[prost(message, repeated, tag = "3")]
    pub deltas: Vec<MessageDelta>,
}

pub struct OutboxQueue {
    queue: VecDeque<PendingMessage>,
    max_retries: u32,
}

impl OutboxQueue {
    pub fn enqueue(&mut self, msg: PendingMessage) {
        self.queue.push_back(msg);
    }

    pub fn pop_for_send(&mut self) -> Option<&mut PendingMessage> {
        self.queue.iter_mut().find(|m| m.status == SendStatus::Pending)
    }

    pub fn mark_acked(&mut self, seq_id: u64) {
        self.queue.retain(|m| m.seq_id != seq_id);
    }
}`
  },
  {
    filename: 'src/ui/fenestra_window.rs',
    title: 'Fenestra Desktop Window & Virtualized List Layout',
    description: 'Direct GPU hardware-accelerated retained rendering with zero virtual-DOM overhead.',
    code: `use fenestra::prelude::*;

pub struct WeChatWindow {
    active_conv_id: Signal<String>,
    messages: Signal<Vec<Message>>,
    unread_total: Memo<usize>,
}

impl Component for WeChatWindow {
    fn render(&self, ctx: &mut FrameContext) -> Element {
        hstack![
            // Sidebar Navigation (64px)
            self.render_nav_sidebar(ctx),
            
            // Conversation List (280px)
            self.render_chat_list(ctx),
            
            // Active Conversation Virtualized Feed (Flexible)
            vstack![
                self.render_chat_header(ctx),
                
                // Virtualized List: Only renders visible ~15 items
                VirtualList::new(&self.messages)
                    .item_height(68.0)
                    .overscan(4)
                    .render_item(|msg| {
                        MessageBubble::new(msg)
                    }),
                    
                self.render_input_toolbar(ctx),
            ].flex(1.0)
        ]
        .background(Color::hex("#f5f5f5"))
        .window_title("WeChat (Rust + Fenestra Native)")
    }
}`
  },
  {
    filename: 'src/sync/worker.rs',
    title: 'Async Sync Worker & Crossbeam Channel Dispatcher',
    description: 'Decoupled background worker running on Tokio runtime with cross-thread lockless messaging.',
    code: `use tokio::time::{sleep, Duration};
use flume::{Receiver, Sender};

pub enum EngineCommand {
    SendMessage(OutboxMsg),
    TriggerSyncPull,
    ChangeNetworkState(NetworkMode),
    CompactionRequest,
}

pub async fn start_background_sync_service(
    cmd_rx: Receiver<EngineCommand>,
    event_tx: Sender<EngineEvent>,
    storage: Arc<RustStorageEngine>,
) {
    let mut sync_key = storage.get_last_sync_key().unwrap_or(0);
    let mut interval = tokio::time::interval(Duration::from_millis(3000));

    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv_async() => {
                match cmd {
                    EngineCommand::SendMessage(msg) => {
                        let seq = sync_key + 1;
                        storage.store_message(&msg.to_message(seq)).unwrap();
                        event_tx.send_async(EngineEvent::MessageStored(seq)).await.ok();
                    }
                    EngineCommand::TriggerSyncPull => {
                        // Incremental sequence pull
                        sync_key = pull_delta_sync(&storage, sync_key).await;
                    }
                    _ => {}
                }
            }
            _ = interval.tick() => {
                // Heartbeat sync check
                sync_key = pull_delta_sync(&storage, sync_key).await;
            }
        }
    }
}`
  }
];
