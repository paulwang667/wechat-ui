//! Mock seed data ported from `src/mock/initialData.ts`.
//! Timestamps are computed relative to `now_ms` so snapshots stay deterministic.

use crate::types::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Test/seed helper: build a [`Message`] with defaults and targeted overrides.
#[derive(Debug, Clone)]
pub struct MsgBuilder {
    pub id: String,
    pub conversation_id: String,
    pub content: String,
    pub sender_id: String,
    pub sender_name: String,
    pub is_self: bool,
    pub message_type: MessageType,
    pub timestamp: i64,
    pub seq_id: i64,
    pub status: SyncStatus,
}

impl Default for MsgBuilder {
    fn default() -> Self {
        Self {
            id: String::new(),
            conversation_id: String::new(),
            content: String::new(),
            sender_id: "user_self".into(),
            sender_name: "Rust Dev (You)".into(),
            is_self: true,
            message_type: MessageType::Text,
            timestamp: now_ms(),
            seq_id: 1,
            status: SyncStatus::Synced,
        }
    }
}

impl MsgBuilder {
    pub fn at(mut self, ts: i64) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn from(mut self, sender_id: &str, sender_name: &str) -> Self {
        self.sender_id = sender_id.into();
        self.sender_name = sender_name.into();
        self.is_self = false;
        self
    }

    pub fn of_type(mut self, ty: MessageType) -> Self {
        self.message_type = ty;
        self
    }

    pub fn with_seq(mut self, seq: i64) -> Self {
        self.seq_id = seq;
        self
    }

    pub fn build(self) -> Message {
        Message {
            id: self.id,
            conversation_id: self.conversation_id,
            sender_id: self.sender_id,
            sender_name: self.sender_name,
            sender_avatar: String::new(),
            is_self: self.is_self,
            message_type: self.message_type,
            content: self.content,
            timestamp: self.timestamp,
            seq_id: self.seq_id,
            status: self.status,
            media_url: None,
            file_name: None,
            file_size: None,
            voice_duration: None,
            code_language: None,
            code_snippet: None,
            quoted_message: None,
            raw_bytes: None,
            compressed_bytes: None,
            is_compressed: false,
        }
    }
}

const AVATAR_BOT: &str = "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=150&auto=format&fit=crop&q=80";
const AVATAR_SELF: &str = "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=150&auto=format&fit=crop&q=80";
const AVATAR_FENESTRA: &str = "https://images.unsplash.com/photo-1633356122544-f134324a6cee?w=150&auto=format&fit=crop&q=80";
const AVATAR_LINUS: &str = "https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=150&auto=format&fit=crop&q=80";
const AVATAR_SARAH: &str = "https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=150&auto=format&fit=crop&q=80";
const AVATAR_WECHAT_TEAM: &str = "https://images.unsplash.com/photo-1550751827-4bd374c3f58b?w=150&auto=format&fit=crop&q=80";
const AVATAR_RUST_CHINA: &str = "https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?w=150&auto=format&fit=crop&q=80";

pub fn current_user() -> Contact {
    Contact {
        id: "user_self".into(),
        name: "Rust Dev (You)".into(),
        alias: Some("Ferris".into()),
        avatar: AVATAR_SELF.into(),
        wechat_id: "rust_rustacean_01".into(),
        region: "Hangzhou, China".into(),
        bio: "Systems Programmer • Rust + Fenestra enthusiast • Zero-cost abstractions".into(),
        category: ContactCategory::Friend,
        online_status: OnlineStatus::Online,
    }
}

pub fn initial_contacts() -> Vec<Contact> {
    vec![
        Contact {
            id: "bot_rust_engine".into(),
            name: "Rust Sync Engine (Bot)".into(),
            alias: Some("Rust Core Bot".into()),
            avatar: AVATAR_BOT.into(),
            wechat_id: "rust_engine_ai".into(),
            region: "Tokyo, Japan".into(),
            bio: "Simulated high-concurrency Rust sync node (Seq ID engine, Zero-Copy RingBuffer)"
                .into(),
            category: ContactCategory::Bot,
            online_status: OnlineStatus::Online,
        },
        Contact {
            id: "contact_fenestra_core".into(),
            name: "Fenestra UI Framework".into(),
            alias: Some("Fenestra Architecture Guild".into()),
            avatar: AVATAR_FENESTRA.into(),
            wechat_id: "fenestra_org".into(),
            region: "Stockholm, Sweden".into(),
            bio: "Ultra-lightweight native desktop GUI framework in Rust. 15MB RAM, 120 FPS."
                .into(),
            category: ContactCategory::Group,
            online_status: OnlineStatus::Online,
        },
        Contact {
            id: "contact_linus".into(),
            name: "Linus Torvalds".into(),
            alias: Some("Linus".into()),
            avatar: AVATAR_LINUS.into(),
            wechat_id: "torvalds_kernel".into(),
            region: "Portland, USA".into(),
            bio: "Talk is cheap. Show me the code.".into(),
            category: ContactCategory::Friend,
            online_status: OnlineStatus::Busy,
        },
        Contact {
            id: "contact_sarah".into(),
            name: "Sarah Chen (Tech Lead)".into(),
            alias: Some("Sarah".into()),
            avatar: AVATAR_SARAH.into(),
            wechat_id: "sarah_arch".into(),
            region: "Shenzhen, China".into(),
            bio: "Optimizing client storage & sync latency. Rust in production.".into(),
            category: ContactCategory::Colleague,
            online_status: OnlineStatus::Online,
        },
        Contact {
            id: "contact_wechat_team".into(),
            name: "WeChat Desktop Team".into(),
            alias: Some("WeChat Official".into()),
            avatar: AVATAR_WECHAT_TEAM.into(),
            wechat_id: "wechat_desktop_official".into(),
            region: "Guangzhou, China".into(),
            bio: "Official WeChat Desktop announcements & performance optimization changelog"
                .into(),
            category: ContactCategory::Service,
            online_status: OnlineStatus::Offline,
        },
        Contact {
            id: "contact_rust_china".into(),
            name: "Rust Systems Dev Community".into(),
            alias: Some("Rust Desktop SIG".into()),
            avatar: AVATAR_RUST_CHINA.into(),
            wechat_id: "rust_china_sig".into(),
            region: "Beijing, China".into(),
            bio: "Rust GUI, cross-platform compilation, sled, redb, and tokio microservices"
                .into(),
            category: ContactCategory::Group,
            online_status: OnlineStatus::Online,
        },
    ]
}

pub fn initial_conversations(now: i64) -> Vec<Conversation> {
    let min = 60_000i64;
    let hour = 3_600_000i64;
    vec![
        Conversation {
            id: "conv_rust_bot".into(),
            name: "Rust Sync Engine (Bot)".into(),
            avatar: AVATAR_BOT.into(),
            is_group: false,
            group_members: vec![],
            last_message: Some(
                "Sync Engine ready. Sequence ID: 1042. Local storage compressed with LZ4 (ratio 68.4%)."
                    .into(),
            ),
            last_message_time: now - 2 * min,
            unread_count: 0,
            is_pinned: true,
            is_muted: false,
            draft: None,
            last_sync_seq: 1042,
        },
        Conversation {
            id: "conv_fenestra_core".into(),
            name: "Fenestra UI Framework".into(),
            avatar: AVATAR_FENESTRA.into(),
            is_group: true,
            group_members: vec![],
            last_message: Some(
                "fenestra-v0.9.4 released: zero-allocation virtualized list rendering ready!"
                    .into(),
            ),
            last_message_time: now - 15 * min,
            unread_count: 2,
            is_pinned: true,
            is_muted: false,
            draft: None,
            last_sync_seq: 890,
        },
        Conversation {
            id: "conv_sarah".into(),
            name: "Sarah Chen (Tech Lead)".into(),
            avatar: AVATAR_SARAH.into(),
            is_group: false,
            group_members: vec![],
            last_message: Some(
                "Can you benchmark the SQLite WAL compaction vs our Rust redb store?".into(),
            ),
            last_message_time: now - 45 * min,
            unread_count: 0,
            is_pinned: false,
            is_muted: false,
            draft: None,
            last_sync_seq: 560,
        },
        Conversation {
            id: "conv_linus".into(),
            name: "Linus Torvalds".into(),
            avatar: AVATAR_LINUS.into(),
            is_group: false,
            group_members: vec![],
            last_message: Some(
                "Good to see you ditching heavy Electron. 15MB RAM baseline is the way.".into(),
            ),
            last_message_time: now - 2 * hour,
            unread_count: 0,
            is_pinned: false,
            is_muted: false,
            draft: None,
            last_sync_seq: 310,
        },
        Conversation {
            id: "conv_rust_china".into(),
            name: "Rust Systems Dev Community".into(),
            avatar: AVATAR_RUST_CHINA.into(),
            is_group: true,
            group_members: vec![],
            last_message: Some(
                "WeChat client with Rust + Fenestra is lightning fast. Zero frame drops!".into(),
            ),
            last_message_time: now - 5 * hour,
            unread_count: 0,
            is_pinned: false,
            is_muted: true,
            draft: None,
            last_sync_seq: 204,
        },
        Conversation {
            id: "conv_wechat_team".into(),
            name: "WeChat Desktop Team".into(),
            avatar: AVATAR_WECHAT_TEAM.into(),
            is_group: false,
            group_members: vec![],
            last_message: Some(
                "Notice: Sync protocol v3 with protobuf delta compaction is enabled.".into(),
            ),
            last_message_time: now - 24 * hour,
            unread_count: 0,
            is_pinned: false,
            is_muted: false,
            draft: None,
            last_sync_seq: 98,
        },
    ]
}

const SYNC_WORKER_SNIPPET: &str = r#"// Rust Tokio Async Incremental Sync Loop
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
}"#;

pub fn initial_messages(now: i64) -> Vec<Message> {
    let min = 60_000i64;
    let hour = 3_600_000i64;
    let m = |id: &str,
                 conv: &str,
                 sender: &str,
                 name: &str,
                 avatar: &str,
                 is_self: bool,
                 ty: MessageType,
                 content: &str,
                 ts: i64,
                 seq: i64| {
        Message {
            id: id.into(),
            conversation_id: conv.into(),
            sender_id: sender.into(),
            sender_name: name.into(),
            sender_avatar: avatar.into(),
            is_self,
            message_type: ty,
            content: content.into(),
            timestamp: ts,
            seq_id: seq,
            status: SyncStatus::Synced,
            media_url: None,
            file_name: None,
            file_size: None,
            voice_duration: None,
            code_language: None,
            code_snippet: None,
            quoted_message: None,
            raw_bytes: None,
            compressed_bytes: None,
            is_compressed: false,
        }
    };

    let bot = ("bot_rust_engine", "Rust Sync Engine (Bot)", AVATAR_BOT);
    let self_user = ("user_self", "Rust Dev (You)", AVATAR_SELF);
    let fen = (
        "contact_fenestra_core",
        "Fenestra Lead Dev",
        AVATAR_FENESTRA,
    );
    let sarah = ("contact_sarah", "Sarah Chen", AVATAR_SARAH);

    vec![
        m("msg_bot_01", "conv_rust_bot", bot.0, bot.1, bot.2, false,
          MessageType::System,
          "⚡ [Rust Native Core] Initialized storage engine (SQLite/redb + LZ4 compression + FTS5 full text index). Base memory: 14.8MB.",
          now - 20 * min, 1039),
        m("msg_bot_02", "conv_rust_bot", self_user.0, self_user.1, self_user.2, true,
          MessageType::Text,
          "Hello! I am testing the new incremental message sync protocol with zero-copy deserialization.",
          now - 18 * min, 1040),
        {
            let mut msg = m("msg_bot_03", "conv_rust_bot", bot.0, bot.1, bot.2, false,
              MessageType::Code,
              "Here is our core Rust Tokio sync worker loop:",
              now - 10 * min, 1041);
            msg.code_language = Some("rust".into());
            msg.code_snippet = Some(SYNC_WORKER_SNIPPET.into());
            msg
        },
        m("msg_bot_04", "conv_rust_bot", bot.0, bot.1, bot.2, false,
          MessageType::Text,
          "Sync Engine ready. Sequence ID: 1042. Local storage compressed with LZ4 (ratio 68.4%). Send me any message to test real-time delta sync!",
          now - 2 * min, 1042),
        m("msg_fen_01", "conv_fenestra_core", fen.0, fen.1, fen.2, false,
          MessageType::Text,
          "Welcome to Fenestra! We just achieved 120 FPS rendering on 4K monitors while keeping CPU usage under 0.8% during active scrolling.",
          now - 30 * min, 888),
        {
            let mut msg = m("msg_fen_02", "conv_fenestra_core", fen.0, fen.1, fen.2, false,
              MessageType::Voice,
              "[Voice Memo: 6 seconds]",
              now - 20 * min, 889);
            msg.voice_duration = Some(6);
            msg
        },
        {
            let mut msg = m("msg_fen_03", "conv_fenestra_core", fen.0, fen.1, fen.2, false,
              MessageType::File,
              "fenestra_benchmark_report_v0.9.4.pdf",
              now - 15 * min, 890);
            msg.file_name = Some("fenestra_benchmark_report_v0.9.4.pdf".into());
            msg.file_size = Some("1.4 MB".into());
            msg
        },
        m("msg_sarah_01", "conv_sarah", sarah.0, sarah.1, sarah.2, false,
          MessageType::Text,
          "Hey! The SQLite WAL auto-checkpointing helped reduce our disk write amplification by 74%.",
          now - 60 * min, 559),
        m("msg_sarah_02", "conv_sarah", sarah.0, sarah.1, sarah.2, false,
          MessageType::Text,
          "Can you benchmark the SQLite WAL compaction vs our Rust redb store?",
          now - 45 * min, 560),
        m("msg_linus_01", "conv_linus", "contact_linus", "Linus Torvalds", AVATAR_LINUS, false,
          MessageType::Text,
          "Good to see you ditching heavy Electron. 15MB RAM baseline is the way. Keep the kernel calls clean.",
          now - 2 * hour, 310),
    ]
}

/// Architecture snippet source code by index (mirrors RUST_ARCHITECTURE_SNIPPETS).
pub fn arch_snippet_code(idx: usize) -> String {
    const SNIPPETS: [&str; 4] = [
        r#"use std::sync::Arc;
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
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = -8000;
            PRAGMA mmap_size = 67108864;

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
        let raw_bytes = serde_json::to_vec(msg)?;
        let compressed = compress_prepend_size(&raw_bytes);
        let crc = crc32fast::hash(&compressed);
        let conn = self.db.write();
        conn.execute(
            "INSERT INTO messages VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![msg.seq_id, msg.conversation_id, msg.sender_id,
                    msg.msg_type as u8, msg.timestamp, compressed, crc],
        )?;
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
}"#,
        r#"use prost::Message as ProstMessage;
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
}"#,
        r##"use fenestra::prelude::*;

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
                VirtualList::new(&self.messages)
                    .item_height(68.0)
                    .overscan(4)
                    .render_item(|msg| MessageBubble::new(msg)),
                self.render_input_toolbar(ctx),
            ].flex(1.0)
        ]
        .background(Color::hex("#f5f5f5"))
        .window_title("WeChat (Rust + Fenestra Native)")
    }
}"##,
        r#"use tokio::time::{sleep, Duration};
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
                        sync_key = pull_delta_sync(&storage, sync_key).await;
                    }
                    _ => {}
                }
            }
            _ = interval.tick() => {
                sync_key = pull_delta_sync(&storage, sync_key).await;
            }
        }
    }
}"#,
    ];
    SNIPPETS.get(idx).copied().unwrap_or("").to_string()
}

/// HH:MM:SS formatting for log timestamps.
pub fn format_clock(ts_ms: i64) -> String {
    let rem = (ts_ms / 1000).rem_euclid(86_400);
    format!(
        "{:02}:{:02}:{:02}",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}
