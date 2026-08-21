//! Storage engine ported from `src/services/rustStorageEngine.ts`.
//! In-memory maps + inverted FTS index + simulated compression accounting.
//! The TS localStorage persistence layer is dropped (native app owns state).

use crate::types::*;
use std::collections::{HashMap, HashSet};

pub struct StorageEngine {
    messages: HashMap<String, Message>,
    conversation_index: HashMap<String, Vec<String>>,
    search_inverted_index: HashMap<String, HashSet<String>>,
    blob_registry: HashMap<String, BlobEntry>,

    total_raw_bytes: u64,
    total_compressed_bytes: u64,
    total_index_bytes: u64,
    total_wal_bytes: u64,
    cache_hits: u64,
    cache_misses: u64,
    last_compaction_time: i64,
}

#[derive(Debug, Clone)]
struct BlobEntry {
    #[allow(dead_code)] // mirrors TS blob registry; kept for export parity
    hash: String,
    size: u64,
    ref_count: u32,
}

pub struct InsertedMessage {
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompactionReport {
    pub freed_bytes: u64,
    pub new_total_bytes: u64,
    pub duration_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StressResult {
    pub duration_ms: f64,
    pub inserted: usize,
}

fn byte_size_of(s: &str) -> u64 {
    // TS `JSON.stringify(msg).length * 2` ≈ 2 bytes per char (UTF-16 assumption).
    s.chars().count() as u64 * 2
}

impl Default for StorageEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            conversation_index: HashMap::new(),
            search_inverted_index: HashMap::new(),
            blob_registry: HashMap::new(),
            total_raw_bytes: 0,
            total_compressed_bytes: 0,
            total_index_bytes: 0,
            total_wal_bytes: 42 * 1024, // baseline WAL size (42KB)
            cache_hits: 142,
            cache_misses: 3,
            last_compaction_time: crate::mock::now_ms() - 1000 * 3600 * 4,
        }
    }

    /// Simulated LZ4/Snappy ratio: code compresses best, media worst.
    fn compression_ratio(message_type: MessageType) -> f64 {
        match message_type {
            MessageType::Code => 0.32,
            MessageType::Text => 0.38,
            _ => 0.45,
        }
    }

    pub fn insert_message(&mut self, mut msg: Message) -> Message {
        let raw = byte_size_of(&msg.content) + 48; // + envelope overhead approximation
        let compressed = ((raw as f64) * Self::compression_ratio(msg.message_type)) as u64;
        let compressed = compressed.max(24);
        msg.raw_bytes = Some(raw);
        msg.compressed_bytes = Some(compressed);
        msg.is_compressed = true;

        let id = msg.id.clone();
        let conv = msg.conversation_id.clone();

        // FTS index over content + code snippet + file name
        let indexable = format!(
            "{} {} {}",
            msg.content,
            msg.code_snippet.as_deref().unwrap_or(""),
            msg.file_name.as_deref().unwrap_or("")
        );
        self.index_message_content(&id, &indexable);

        // Blob dedup for attachments
        if let Some(blob_key) = msg.media_url.as_ref().or(msg.file_name.as_ref()) {
            let key = blob_key.clone();
            let size = raw.max(1024);
            let entry = self.blob_registry.entry(key).or_insert_with(|| BlobEntry {
                hash: format!("sha256_{:x}", simple_hash(blob_key)),

                size,
                ref_count: 0,
            });
            entry.ref_count += 1;
        }

        self.total_raw_bytes += raw;
        self.total_compressed_bytes += compressed;
        self.total_index_bytes += 32; // index entry cost
        self.total_wal_bytes += 48; // append-only WAL cost

        self.conversation_index
            .entry(conv)
            .or_default()
            .push(id.clone());
        self.messages.insert(id, msg.clone());
        msg
    }

    /// Direct read-only accessor by id.

    /// Read-only listing (does not touch cache-hit stats).
    pub fn peek_messages_by_conversation(&self, conversation_id: &str) -> Vec<Message> {
        let mut out: Vec<Message> = self
            .conversation_index
            .get(conversation_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.messages.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default();
        out.sort_by_key(|m| m.timestamp);
        out
    }

    /// Direct read-only accessor by id.
    pub fn get_message(&self, id: &str) -> Option<Message> {
        self.messages.get(id).cloned()
    }

    fn index_message_content(&mut self, msg_id: &str, text: &str) {
        if text.trim().is_empty() {
            return;
        }
        for token in tokenize(text) {
            self.search_inverted_index
                .entry(token)
                .or_default()
                .insert(msg_id.to_string());
        }
    }

    pub fn get_messages_by_conversation(&mut self, conversation_id: &str) -> Vec<Message> {
        self.cache_hits += 1;
        let mut out: Vec<Message> = self
            .conversation_index
            .get(conversation_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.messages.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default();
        out.sort_by_key(|m| m.timestamp);
        out
    }

    pub fn search_messages(
        &mut self,
        query: &str,
        conversation_id: Option<&str>,
    ) -> (Vec<Message>, f64) {
        let start = std::time::Instant::now();
        let clean = query.to_lowercase();
        let clean = clean.trim();
        if clean.is_empty() {
            return (vec![], 0.0);
        }

        let tokens: Vec<&str> = clean.split_whitespace().collect();
        let mut candidate_ids: HashSet<String> = HashSet::new();
        for token in &tokens {
            for (key, ids) in &self.search_inverted_index {
                if key.contains(token) {
                    candidate_ids.extend(ids.iter().cloned());
                }
            }
        }

        let mut results: Vec<Message> = candidate_ids
            .into_iter()
            .filter_map(|id| self.messages.get(&id).cloned())
            .filter(|m| conversation_id.is_none_or(|c| m.conversation_id == c))
            .collect();

        // Fallback direct scan when the index misses (tiny/single-char queries).
        if results.is_empty() {
            for msg in self.messages.values() {
                if let Some(c) = conversation_id {
                    if msg.conversation_id != c {
                        continue;
                    }
                }
                let lc = msg.content.to_lowercase();
                let hit = lc.contains(clean)
                    || msg
                        .file_name
                        .as_deref()
                        .is_some_and(|f| f.to_lowercase().contains(clean))
                    || msg
                        .code_snippet
                        .as_deref()
                        .is_some_and(|s| s.to_lowercase().contains(clean));
                if hit {
                    results.push(msg.clone());
                }
            }
        }

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        (results, (ms * 100.0).round() / 100.0)
    }

    pub fn update_message_status(&mut self, id: &str, status: SyncStatus, seq_id: Option<i64>) {
        if let Some(msg) = self.messages.get_mut(id) {
            msg.status = status;
            if let Some(seq) = seq_id {
                msg.seq_id = seq;
            }
        }
    }

    pub fn clear_conversation(&mut self, conv_id: &str) {
        if let Some(ids) = self.conversation_index.remove(conv_id) {
            for id in ids {
                if let Some(msg) = self.messages.remove(&id) {
                    self.total_raw_bytes =
                        self.total_raw_bytes.saturating_sub(msg.raw_bytes.unwrap_or(0));
                    self.total_compressed_bytes = self
                        .total_compressed_bytes
                        .saturating_sub(msg.compressed_bytes.unwrap_or(0));
                }
            }
        }
    }

    pub fn batch_insert_stress_test(&mut self, count: usize, conv_id: &str) -> StressResult {
        let start = std::time::Instant::now();
        let sample_sentences = [
            "Rust memory safety without garbage collection yields predictable tail latency.",
            "Fenestra rendering pipeline achieves 120 FPS by compiling UI layout trees directly to GPU vertices.",
            "Incremental sync sequence confirmed. Vector clock synchronized across peer nodes.",
            "Zero-copy serialization with rkyv reduces deserialization overhead to exactly 0ns.",
            "SQLite WAL auto-checkpoint pruned 14MB of fragmented pages.",
            "Benchmarking message sync throughput: 45,000 msgs/sec on single core Tokio worker.",
        ];
        let now = crate::mock::now_ms();
        for i in 0..count {
            let text = format!(
                "{} [Stress Test Seq #{}]",
                sample_sentences[i % sample_sentences.len()],
                i + 1
            );
            let is_self = i % 2 != 0;
            let msg = Message {
                id: format!("stress_{}_{}", now, i),
                conversation_id: conv_id.into(),
                sender_id: if is_self { "user_self" } else { "bot_rust_engine" }.into(),
                sender_name: if is_self {
                    "Rust Dev (You)"
                } else {
                    "Rust Sync Engine (Bot)"
                }
                .into(),
                sender_avatar: String::new(),
                is_self,
                message_type: MessageType::Text,
                content: text,
                timestamp: now - ((count - i) as i64) * 1000,
                seq_id: 2000 + i as i64,
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
            self.insert_message(msg);
        }
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        StressResult {
            duration_ms: (ms * 100.0).round() / 100.0,
            inserted: count,
        }
    }

    pub fn vacuum_and_compact(&mut self) -> CompactionReport {
        let start = std::time::Instant::now();
        let _before =
            self.total_compressed_bytes + self.total_wal_bytes + self.total_index_bytes;

        // Prune WAL into primary store
        let freed_wal = self.total_wal_bytes * 3 / 4;
        self.total_wal_bytes = (self.total_wal_bytes - freed_wal).max(8192);

        // Defragment B-tree index
        let freed_index = self.total_index_bytes / 5;
        self.total_index_bytes = (self.total_index_bytes - freed_index).max(1024);

        let freed_bytes = freed_wal + freed_index;
        let new_total_bytes =
            self.total_compressed_bytes + self.total_wal_bytes + self.total_index_bytes;
        self.last_compaction_time = crate::mock::now_ms();

        let ms = start.elapsed().as_secs_f64() * 1000.0;
        CompactionReport {
            freed_bytes,
            new_total_bytes,
            duration_ms: (ms * 100.0).round() / 100.0,
        }
    }

    #[allow(clippy::comparison_chain)]
    pub fn get_metrics(&self) -> StorageMetrics {
        let total_msgs = self.messages.len() as u64;
        let total_convs = self.conversation_index.len() as u64;
        let raw = if self.total_raw_bytes == 0 {
            24_000
        } else {
            self.total_raw_bytes
        };
        let comp = if self.total_compressed_bytes == 0 {
            8_200
        } else {
            self.total_compressed_bytes
        };
        let saved = raw.saturating_sub(comp);
        let ratio = if raw > 0 {
            ((1.0 - comp as f64 / raw as f64) * 100.0 * 10.0).round() / 10.0
        } else {
            68.4
        };

        let mut blob_saved = 0u64;
        let mut blob_dedup_count = 0u64;
        for b in self.blob_registry.values() {
            if b.ref_count > 1 {
                blob_dedup_count += (b.ref_count - 1) as u64;
                blob_saved += (b.ref_count - 1) as u64 * b.size;
            }
        }

        let total_cache = self.cache_hits + self.cache_misses;
        let cache_hit_ratio = if total_cache > 0 {
            (self.cache_hits as f64 / total_cache as f64 * 100.0 * 10.0).round() / 10.0
        } else {
            98.2
        };

        StorageMetrics {
            total_messages: total_msgs,
            total_conversations: total_convs,
            raw_storage_bytes: raw,
            compressed_storage_bytes: comp,
            saved_storage_bytes: saved,
            compression_ratio: ratio,
            index_size_bytes: if self.total_index_bytes == 0 {
                4096
            } else {
                self.total_index_bytes
            },
            wal_log_size_bytes: self.total_wal_bytes,
            blob_deduplicated_count: blob_dedup_count,
            blob_saved_bytes: blob_saved,
            active_memory_cache_bytes: comp * 2 / 5, // 40% hot in memory ringbuffer
            cache_hit_ratio,
            last_compaction_time: self.last_compaction_time,
        }
    }

    pub fn export_database_json(&self) -> String {
        let mut all: Vec<&Message> = self.messages.values().collect();
        all.sort_by_key(|m| m.seq_id);
        let doc = serde_json::json!({
            "schema_version": "fenestra_wechat_v1_rust",
            "exported_at": chrono_iso_now(),
            "messages_count": all.len(),
            "metrics": self.get_metrics(),
            "messages": all,
        });
        serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".into())
    }

    pub fn import_database_json(&mut self, json: &str) -> Result<usize, String> {
        let parsed: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Failed to parse JSON: {e}"))?;
        let arr = parsed
            .get("messages")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Invalid database payload: missing messages array".to_string())?;
        let mut count = 0usize;
        for v in arr {
            let m: Message = match serde_json::from_value(v.clone()) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if !m.id.is_empty() && !m.conversation_id.is_empty() {
                self.insert_message(m);
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

/// TS: `blobKey.split('').reduce((a,b)=>((a<<5)-a)+b.charCodeAt(0),0)` — djb2-ish.
fn simple_hash(s: &str) -> u64 {
    let mut h: i64 = 0;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as i64);
    }
    h.unsigned_abs()
}

/// Lowercase, strip non-word/non-CJK, split on whitespace.
fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let cleaned: String = lower
        .chars()
        .map(|c| {
            let keep = c.is_ascii_alphanumeric()
                || c == '_'
                || ('\u{4e00}'..='\u{9fa5}').contains(&c);
            if keep {
                c
            } else {
                ' '
            }
        })
        .collect();
    cleaned.split_whitespace().map(String::from).collect()
}

fn chrono_iso_now() -> String {
    // Minimal ISO-8601 without pulling chrono: seconds precision from epoch math.
    let secs = crate::mock::now_ms() / 1000;
    let days = secs / 86_400;
    let rem = secs % 86_400;
    // Civil-from-days algorithm (Howard Hinnant) — good enough for a demo export stamp.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.000Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}
