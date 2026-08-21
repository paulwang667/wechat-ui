//! Domain types ported 1:1 from `src/types/wechat.ts`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Image,
    Voice,
    File,
    Code,
    System,
    Quote,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuotedMessage {
    pub sender_name: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    Sending,
    Synced,
    Failed,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub sender_avatar: String,
    pub is_self: bool,
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub content: String,
    /// Milliseconds since Unix epoch.
    pub timestamp: i64,
    /// Incremental sequence ID for sync.
    pub seq_id: i64,
    pub status: SyncStatus,

    // Optional rich content
    #[serde(default)]
    pub media_url: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub file_size: Option<String>,
    /// Seconds.
    #[serde(default)]
    pub voice_duration: Option<u32>,
    #[serde(default)]
    pub code_language: Option<String>,
    #[serde(default)]
    pub code_snippet: Option<String>,
    #[serde(default)]
    pub quoted_message: Option<QuotedMessage>,

    // Storage & compression metadata
    #[serde(default)]
    pub raw_bytes: Option<u64>,
    #[serde(default)]
    pub compressed_bytes: Option<u64>,
    #[serde(default)]
    pub is_compressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactCategory {
    Friend,
    Colleague,
    Group,
    Bot,
    Service,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlineStatus {
    Online,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub alias: Option<String>,
    pub avatar: String,
    pub wechat_id: String,
    pub region: String,
    pub bio: String,
    pub category: ContactCategory,
    pub online_status: OnlineStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub is_group: bool,
    #[serde(default)]
    pub group_members: Vec<Contact>,
    #[serde(default)]
    pub last_message: Option<String>,
    pub last_message_time: i64,
    pub unread_count: u32,
    pub is_pinned: bool,
    pub is_muted: bool,
    #[serde(default)]
    pub draft: Option<String>,
    pub last_sync_seq: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncLogType {
    PullReq,
    PullResp,
    PushMsg,
    Ack,
    ConflictResolve,
    Compact,
    Heartbeat,
}

impl SyncLogType {
    /// Wire label matching the TS constants (`PULL_REQ`, …).
    pub fn as_str(self) -> &'static str {
        match self {
            SyncLogType::PullReq => "PULL_REQ",
            SyncLogType::PullResp => "PULL_RESP",
            SyncLogType::PushMsg => "PUSH_MSG",
            SyncLogType::Ack => "ACK",
            SyncLogType::ConflictResolve => "CONFLICT_RESOLVE",
            SyncLogType::Compact => "COMPACT",
            SyncLogType::Heartbeat => "HEARTBEAT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncLogEntry {
    pub id: String,
    pub timestamp: i64,
    #[serde(rename = "type")]
    pub log_type: SyncLogType,
    pub seq: i64,
    pub details: String,
    pub latency_ms: f64,
    pub bytes_transferred: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub total_messages: u64,
    pub total_conversations: u64,
    pub raw_storage_bytes: u64,
    pub compressed_storage_bytes: u64,
    pub saved_storage_bytes: u64,
    /// e.g. 0.68 for 68%.
    pub compression_ratio: f64,
    pub index_size_bytes: u64,
    pub wal_log_size_bytes: u64,
    pub blob_deduplicated_count: u64,
    pub blob_saved_bytes: u64,
    pub active_memory_cache_bytes: u64,
    /// e.g. 0.984 for 98.4%.
    pub cache_hit_ratio: f64,
    pub last_compaction_time: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    Optimal,
    Wifi,
    Cellular4g,
    HighLatency,
    PacketLoss,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NetworkCondition {
    pub mode: NetworkMode,
    pub latency_ms: f64,
    /// 0.0 to 1.0.
    pub packet_loss_rate: f64,
    pub bandwidth_kbps: f64,
}

impl Default for NetworkCondition {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Optimal,
            latency_ms: 12.0,
            packet_loss_rate: 0.0,
            bandwidth_kbps: 50_000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmarkResult {
    pub message_count: u64,
    pub insert_time_ms: f64,
    pub insert_rate_msg_per_sec: f64,
    pub fts_search_time_ms: f64,
    pub scroll_fps: f64,
    pub memory_usage_mb: f64,
    pub fenestra_render_time_ms: f64,
}
