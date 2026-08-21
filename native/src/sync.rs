//! Sync engine ported from `src/services/syncEngine.ts`.
//! Pure state machine: no timers, no threads. The UI layer drives delivery
//! through `tick()` calls (fenestra `Sub::every` in production, pump clock in tests).

use crate::storage::StorageEngine;
use crate::types::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum SyncEvent {
    MessageSaved(Message),
    IncomingMessage(Message),
    MessageStatusChanged { id: String, status: SyncStatus },
    NetworkChanged(NetworkCondition),
    SyncCompleted { seq: i64 },
    LogAdded(SyncLogEntry),
}

pub struct SyncEngine {
    storage: StorageEngine,
    current_seq: i64,
    network: NetworkCondition,
    outbox: VecDeque<Message>,
    sync_logs: Vec<SyncLogEntry>,
    /// Pending deliveries scheduled at a future virtual timestamp (ms).
    scheduled: Vec<(i64, ScheduledAction)>,
    log_counter: u64,
    /// Virtual clock (ms). Advanced by `tick`; initialized from the wall clock.
    clock_ms: i64,
}

enum ScheduledAction {
    DeliverOutgoing(String), // message id
    BotReply(String, String, String), // conv_id, sender_id, sender_name
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            storage: StorageEngine::new(),
            current_seq: 1042,
            network: NetworkCondition::default(),
            outbox: VecDeque::new(),
            sync_logs: Vec::new(),
            scheduled: Vec::new(),
            log_counter: 0,
            clock_ms: crate::mock::now_ms(),
        }
    }

    /// Current virtual clock (ms).
    pub fn now(&self) -> i64 {
        self.clock_ms
    }

    /// Read access to the underlying storage engine.
    pub fn storage(&self) -> &StorageEngine {
        &self.storage
    }

    /// Mutable access for seeding and maintenance operations.
    pub fn storage_mut(&mut self) -> &mut StorageEngine {
        &mut self.storage
    }

    pub fn current_seq(&self) -> i64 {
        self.current_seq
    }

    pub fn network(&self) -> NetworkCondition {
        self.network
    }

    pub fn outbox(&self) -> Vec<Message> {
        self.outbox.iter().cloned().collect()
    }

    pub fn sync_logs(&self) -> Vec<SyncLogEntry> {
        self.sync_logs.clone()
    }

    pub fn set_network_condition(&mut self, condition: PartialNetwork) -> Vec<SyncEvent> {
        if let Some(mode) = condition.mode {
            self.network.mode = mode;
        }
        if let Some(latency) = condition.latency_ms {
            self.network.latency_ms = latency;
        }
        if let Some(loss) = condition.packet_loss_rate {
            self.network.packet_loss_rate = loss;
        }
        if let Some(bw) = condition.bandwidth_kbps {
            self.network.bandwidth_kbps = bw;
        }

        let mut events = vec![SyncEvent::NetworkChanged(self.network)];
        events.push(self.add_log(
            SyncLogType::Heartbeat,
            self.current_seq,
            format!(
                "Network mode set to [{}]: Latency {}ms, Loss {}%",
                mode_label(self.network.mode),
                self.network.latency_ms,
                (self.network.packet_loss_rate * 100.0) as i64
            ),
            self.network.latency_ms,
            64,
            true,
        ));

        // Restored connectivity → flush outbox
        if self.network.mode != NetworkMode::Offline && !self.outbox.is_empty() {
            events.extend(self.flush_outbox());
        }
        events
    }

    /// Deterministic send. `rng` supplies the packet-loss roll (0.0..1.0);
    /// pass 1.0 to force loss-free behavior in tests.
    pub fn send_message(
        &mut self,
        msg: Message,
        now_ms: i64,
        loss_roll: f64,
    ) -> Vec<SyncEvent> {
        let is_offline = self.network.mode == NetworkMode::Offline;
        let is_packet_lost = loss_roll < self.network.packet_loss_rate;

        self.current_seq += 1;
        let next_seq = self.current_seq;
        let initial_status = if is_offline {
            SyncStatus::Pending
        } else {
            SyncStatus::Sending
        };

        let full_msg = Message {
            id: if msg.id.is_empty() {
                self.log_counter += 1;
                format!("msg_out_{}_{}", now_ms, self.log_counter)
            } else {
                msg.id
            },
            seq_id: next_seq,
            status: initial_status,
            ..msg
        };

        let stored = self.storage.insert_message(full_msg.clone());
        let mut events = vec![SyncEvent::MessageSaved(stored.clone())];

        if is_offline {
            self.outbox.push_back(stored.clone());
            events.push(self.add_log(
                SyncLogType::PushMsg,
                next_seq,
                format!("[OFFLINE] Message queued in Outbox (Seq #{next_seq})"),
                0.0,
                stored.compressed_bytes.unwrap_or(64),
                false,
            ));
            return events;
        }

        // Network delivery simulation
        events.push(self.add_log(
            SyncLogType::PushMsg,
            next_seq,
            format!("Sending protobuf delta payload to Sync Gate (Seq #{next_seq})"),
            self.network.latency_ms,
            stored.compressed_bytes.unwrap_or(64),
            !is_packet_lost,
        ));

        let deliver_at = now_ms + (self.network.latency_ms.max(20.0)) as i64;
        if is_packet_lost {
            self.scheduled.push((deliver_at, ScheduledAction::DeliverOutgoing(stored.id.clone())));
        } else {
            self.scheduled.push((deliver_at, ScheduledAction::DeliverOutgoing(stored.id.clone())));
        }
        events
    }

    /// Advance virtual time; fires every scheduled action whose deadline passed.
    /// Returns all resulting events in order.
    pub fn tick(&mut self, now_ms: i64) -> Vec<SyncEvent> {
        if now_ms > self.clock_ms {
            self.clock_ms = now_ms;
        }
        let due: Vec<_> = {
            let mut due_idx = Vec::new();
            for (i, (at, _)) in self.scheduled.iter().enumerate() {
                if *at <= now_ms {
                    due_idx.push(i);
                }
            }
            due_idx.reverse();
            due_idx.into_iter().map(|i| self.scheduled.remove(i).1).collect()
        };
        let mut events = Vec::new();
        for action in due {
            match action {
                ScheduledAction::DeliverOutgoing(id) => {
                    events.extend(self.deliver_outgoing(&id));
                }
                ScheduledAction::BotReply(conv, sender_id, sender_name) => {
                    events.extend(self.bot_reply(&conv, &sender_id, &sender_name));
                }
            }
        }
        events
    }

    fn deliver_outgoing(&mut self, id: &str) -> Vec<SyncEvent> {
        let Some(msg) = self.storage_message(id) else {
            return vec![];
        };
        let seq = msg.seq_id;
        // Re-roll packet loss on each delivery attempt via deterministic hash of id+attempt count.
        let attempt_roll = pseudo_random(id, seq as u64);
        let lost = attempt_roll < self.network.packet_loss_rate;

        if lost {
            self.storage.update_message_status(id, SyncStatus::Failed, None);
            let mut failed = msg.clone();
            failed.status = SyncStatus::Failed;
            self.outbox.push_back(failed.clone());
            let mut events = vec![SyncEvent::MessageStatusChanged {
                id: id.to_string(),
                status: SyncStatus::Failed,
            }];
            events.push(self.add_log(
                SyncLogType::PushMsg,
                seq,
                "Packet drop simulated! Retrying via Tokio async exponential backoff...".into(),
                self.network.latency_ms,
                0,
                false,
            ));
            // Auto retry after 2.5s
            self.scheduled.push((
                self.clock_ms + 2500,
                ScheduledAction::DeliverOutgoing(id.to_string()),
            ));
            return events;
        }

        self.storage.update_message_status(id, SyncStatus::Synced, None);
        let mut events = vec![SyncEvent::MessageStatusChanged {
            id: id.to_string(),
            status: SyncStatus::Synced,
        }];
        events.push(self.add_log(
            SyncLogType::Ack,
            seq,
            format!("Server ACK received for Seq #{seq}. Delta confirmed across cluster."),
            (self.network.latency_ms * 1.2).floor(),
            32,
            true,
        ));
        // Simulated intelligent response for bot/peer conversations
        let reply_target = bot_reply_for(&msg.conversation_id);
        if let Some((conv, sender_id, sender_name)) = reply_target {
            let reply_at = self.clock_ms + 700 + (pseudo_random_u64(id) % 600) as i64;
            self.scheduled
                .push((reply_at, ScheduledAction::BotReply(conv, sender_id, sender_name)));
        }
        events
    }

    fn bot_reply(&mut self, conv_id: &str, sender_id: &str, sender_name: &str) -> Vec<SyncEvent> {
        let replies = [
            "⚡ [Rust Engine ACK] Received. In-memory ring buffer synchronized in 0.04ms.",
            "📦 Stored with LZ4. Compression ratio is currently 68.2%. Zero-copy buffer pointer updated.",
            "🚀 Sync sequence monotonic increment confirmed.",
            "💡 Tip: You can switch network condition to \"Packet Loss\" or \"High Latency\" in the Diagnostics tab to test outbox auto-recovery!",
        ];
        let pick = pseudo_random_u64(conv_id) as usize % replies.len();
        let content = replies[pick].to_string();
        let avatar = crate::mock::initial_contacts()
            .into_iter()
            .find(|c| c.id == sender_id)
            .map(|c| c.avatar)
            .unwrap_or_default();
        self.receive_incoming(IncomingMessage {
            conversation_id: conv_id.into(),
            sender_id: sender_id.into(),
            sender_name: sender_name.into(),
            sender_avatar: avatar,
            is_self: false,
            message_type: MessageType::Text,
            content,
        })
    }

    pub fn flush_outbox(&mut self) -> Vec<SyncEvent> {
        if self.network.mode == NetworkMode::Offline || self.outbox.is_empty() {
            return vec![];
        }
        let pending: Vec<Message> = self.outbox.drain(..).collect();
        let n = pending.len();

        let mut events = vec![self.add_log(
            SyncLogType::PullReq,
            self.current_seq,
            format!("Flushing {n} outbox message(s)..."),
            self.network.latency_ms,
            n as u64 * 96,
            true,
        )];

        for msg in pending {
            self.storage
                .update_message_status(&msg.id, SyncStatus::Synced, None);
            events.push(SyncEvent::MessageStatusChanged {
                id: msg.id.clone(),
                status: SyncStatus::Synced,
            });
            events.push(self.add_log(
                SyncLogType::Ack,
                msg.seq_id,
                format!("Outbox message Seq #{} synced successfully.", msg.seq_id),
                self.network.latency_ms,
                32,
                true,
            ));
            if let Some((conv, sender_id, sender_name)) = bot_reply_for(&msg.conversation_id) {
                let reply_at = self.clock_ms + 700;
                self.scheduled
                    .push((reply_at, ScheduledAction::BotReply(conv, sender_id, sender_name)));
            }
        }
        events
    }

    pub fn receive_incoming(&mut self, msg: IncomingMessage) -> Vec<SyncEvent> {
        self.current_seq += 1;
        let next_seq = self.current_seq;
        let incoming = Message {
            id: format!("msg_in_{}_{}", self.clock_ms, self.log_counter),
            conversation_id: msg.conversation_id,
            sender_id: msg.sender_id,
            sender_name: msg.sender_name,
            sender_avatar: msg.sender_avatar,
            is_self: msg.is_self,
            message_type: msg.message_type,
            content: msg.content.clone(),
            timestamp: self.clock_ms,
            seq_id: next_seq,
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
        };
        self.log_counter += 1;

        let stored = self.storage.insert_message(incoming.clone());
        let preview: String = incoming.content.chars().take(32).collect();
        let events = vec![self.add_log(
            SyncLogType::PullResp,
            next_seq,
            format!("Incremental Delta push arrived: \"{preview}...\" from {}", incoming.sender_name),
            self.network.latency_ms,
            stored.compressed_bytes.unwrap_or(80),
            true,
        ), SyncEvent::IncomingMessage(stored)];
        events
    }

    /// Heartbeat — wire to `Sub::every(15s)`; skip when offline.
    pub fn heartbeat(&mut self) -> Option<SyncEvent> {
        if self.network.mode == NetworkMode::Offline {
            return None;
        }
        Some(self.add_log(
            SyncLogType::Heartbeat,
            self.current_seq,
            format!(
                "Sync Keepalive ping • Client Seq #{} in sync with remote gateway",
                self.current_seq
            ),
            self.network.latency_ms,
            16,
            true,
        ))
    }

    /// Returns (events, completion_delay_ms) — schedule `tick` after the delay.
    pub fn trigger_manual_sync(&mut self) -> (Vec<SyncEvent>, i64) {
        let events = vec![self.add_log(
            SyncLogType::PullReq,
            self.current_seq,
            format!("Manual sync pull requested (SyncKey: {})", self.current_seq),
            self.network.latency_ms,
            64,
            true,
        )];
        let delay = (self.network.latency_ms * 2.0).max(50.0) as i64;
        (events, delay)
    }

    pub fn complete_manual_sync(&mut self) -> Vec<SyncEvent> {
        vec![
            self.add_log(
                SyncLogType::PullResp,
                self.current_seq,
                "Sync response: 0 delta conflicts. Local storage up to date.".into(),
                self.network.latency_ms,
                32,
                true,
            ),
            SyncEvent::SyncCompleted {
                seq: self.current_seq,
            },
        ]
    }

    fn storage_message(&self, id: &str) -> Option<Message> {
        self.storage.get_message(id)
    }


    fn add_log(
        &mut self,
        log_type: SyncLogType,
        seq: i64,
        details: String,
        latency_ms: f64,
        bytes_transferred: u64,
        success: bool,
    ) -> SyncEvent {
        self.log_counter += 1;
        let entry = SyncLogEntry {
            id: format!("log_{}_{}", self.clock_ms, self.log_counter),
            timestamp: self.clock_ms,
            log_type,
            seq,
            details,
            latency_ms,
            bytes_transferred,
            success,
        };
        self.sync_logs.insert(0, entry.clone());
        if self.sync_logs.len() > 100 {
            self.sync_logs.pop();
        }
        SyncEvent::LogAdded(entry)
    }
}

/// Loose network patch object mirroring TS `Partial<NetworkCondition>`.
#[derive(Debug, Clone, Copy, Default)]
pub struct PartialNetwork {
    pub mode: Option<NetworkMode>,
    pub latency_ms: Option<f64>,
    pub packet_loss_rate: Option<f64>,
    pub bandwidth_kbps: Option<f64>,
}

/// Incoming payload: everything except id/timestamp/seq/status.
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub sender_avatar: String,
    pub is_self: bool,
    pub message_type: MessageType,
    pub content: String,
}


fn mode_label(mode: NetworkMode) -> &'static str {
    match mode {
        NetworkMode::Optimal => "OPTIMAL",
        NetworkMode::Wifi => "WIFI",
        NetworkMode::Cellular4g => "CELLULAR_4G",
        NetworkMode::HighLatency => "HIGH_LATENCY",
        NetworkMode::PacketLoss => "PACKET_LOSS",
        NetworkMode::Offline => "OFFLINE",
    }
}

/// Which conversations auto-reply, and as whom.
fn bot_reply_for(conv_id: &str) -> Option<(String, String, String)> {
    match conv_id {
        "conv_rust_bot" => Some((
            conv_id.into(),
            "bot_rust_engine".into(),
            "Rust Sync Engine (Bot)".into(),
        )),
        "conv_fenestra_core" => Some((
            conv_id.into(),
            "contact_fenestra_core".into(),
            "Fenestra Lead Dev".into(),
        )),
        "conv_sarah" => Some((
            conv_id.into(),
            "contact_sarah".into(),
            "Sarah Chen (Tech Lead)".into(),
        )),
        _ => None,
    }
}

/// Deterministic pseudo-random in [0,1) derived from a string + salt.
fn pseudo_random(s: &str, salt: u64) -> f64 {
    let mut h = salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01B3);
    }
    (h >> 11) as f64 / (1u64 << 53) as f64
}

fn pseudo_random_u64(s: &str) -> u64 {
    (pseudo_random(s, 0xDEADBEEF) * 1e9) as u64
}
