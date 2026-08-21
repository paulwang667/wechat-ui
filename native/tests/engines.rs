//! Engine tests: storage + sync ported behavior must match the TS semantics.

use wechat_fui_native::mock;
use wechat_fui_native::storage::StorageEngine;
use wechat_fui_native::sync::{IncomingMessage, PartialNetwork, SyncEngine, SyncEvent};
use wechat_fui_native::types::{NetworkMode, SyncStatus};
use wechat_fui_native::types::{MessageType, NetworkCondition};

fn text_msg(id: &str, conv: &str, content: &str) -> mock::MsgBuilder {
    mock::MsgBuilder {
        id: id.into(),
        conversation_id: conv.into(),
        content: content.into(),
        ..Default::default()
    }
}

#[test]
fn insert_enriches_compression_metadata() {
    let mut eng = StorageEngine::new();
    let msg = text_msg("m1", "c1", "hello world");
    let stored = eng.insert_message(msg.build());
    assert!(stored.raw_bytes.unwrap() > 0);
    assert!(stored.compressed_bytes.unwrap() < stored.raw_bytes.unwrap());
    assert!(stored.is_compressed);
    assert_eq!(eng.message_count(), 1);
}

#[test]
fn conversation_listing_sorted_by_timestamp() {
    let mut eng = StorageEngine::new();
    eng.insert_message(text_msg("a", "c1", "first").at(100).build());
    eng.insert_message(text_msg("b", "c1", "second").at(50).build());
    eng.insert_message(text_msg("c", "c1", "third").at(200).build());
    let msgs = eng.get_messages_by_conversation("c1");
    let ts: Vec<i64> = msgs.iter().map(|m| m.timestamp).collect();
    assert_eq!(ts, vec![50, 100, 200]);
}

#[test]
fn fts_search_finds_indexed_terms() {
    let mut eng = StorageEngine::new();
    eng.insert_message(
        text_msg("a", "c1", "Fenestra rendering achieves 120 FPS").build(),
    );
    eng.insert_message(text_msg("b", "c1", "unrelated content").build());

    let (results, _ms) = eng.search_messages("fenestra", None);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "a");

    // substring match against index keys
    let (results2, _) = eng.search_messages("render", Some("c1"));
    assert_eq!(results2.len(), 1);
    assert_eq!(results2[0].id, "a");
}

#[test]
fn clear_conversation_removes_all_and_updates_metrics() {
    let mut eng = StorageEngine::new();
    eng.insert_message(text_msg("a", "c1", "one").build());
    eng.insert_message(text_msg("b", "c1", "two").build());
    eng.insert_message(text_msg("c", "c2", "keep").build());
    assert_eq!(eng.get_metrics().total_messages, 3);

    eng.clear_conversation("c1");
    assert_eq!(eng.get_metrics().total_messages, 1);
    assert_eq!(eng.get_metrics().total_conversations, 1);
}

#[test]
fn stress_insert_bulk() {
    let mut eng = StorageEngine::new();
    let res = eng.batch_insert_stress_test(2000, "conv_rust_bot");
    assert_eq!(res.inserted, 2000);
    assert_eq!(eng.message_count(), 2000);
    // 6 sample sentences cycle; all indexed
    let (hits, _) = eng.search_messages("stress", Some("conv_rust_bot"));
    assert_eq!(hits.len(), 2000);
}

#[test]
fn compaction_reduces_wal_and_index() {
    let mut eng = StorageEngine::new();
    let before_wal = eng.get_metrics().wal_log_size_bytes;
    let report = eng.vacuum_and_compact();
    assert!(report.freed_bytes > 0);
    let after_wal = eng.get_metrics().wal_log_size_bytes;
    assert!(after_wal < before_wal);
}

#[test]
fn export_import_roundtrip() {
    let mut eng = StorageEngine::new();
    eng.insert_message(text_msg("a", "c1", "payload one").build());
    eng.insert_message(text_msg("b", "c2", "payload two").build());
    let json = eng.export_database_json();

    let mut eng2 = StorageEngine::new();
    let count = eng2.import_database_json(&json).expect("import ok");
    assert_eq!(count, 2);
    assert_eq!(eng2.message_count(), 2);
}

#[test]
fn import_rejects_invalid_payload() {
    let mut eng = StorageEngine::new();
    assert!(eng.import_database_json("not json").is_err());
    assert!(eng
        .import_database_json(r#"{"schema_version":"x"}"#)
        .is_err()); // missing messages array
}

// ---------- Sync engine ----------

fn offline_engine() -> SyncEngine {
    let mut e = SyncEngine::new();
    e.set_network_condition(PartialNetwork {
        mode: Some(NetworkMode::Offline),
        ..Default::default()
    });
    e
}

#[test]
fn offline_send_queues_in_outbox_as_pending() {
    let mut e = offline_engine();
    let events = e.send_message(text_msg("", "conv_rust_bot", "hi").build(), 1000, 1.0);
    // MessageSaved + NetworkChanged(from set_network) ordering: send returns [saved, log]
    assert!(events
        .iter()
        .any(|ev| matches!(ev, wechat_fui_native::sync::SyncEvent::MessageSaved(_))));
    assert_eq!(e.outbox().len(), 1);
    let msg = &e.outbox()[0];
    assert_eq!(msg.status, SyncStatus::Pending);
    assert_eq!(e.current_seq(), 1043); // 1042 + 1
}

#[test]
fn online_send_delivers_after_latency_then_acks() {
    let mut e = SyncEngine::new(); // optimal: latency 12ms, no loss
    let t0 = e.now();
    e.send_message(text_msg("", "conv_linus", "hi").build(), t0, 1.0);

    // tick before deadline: nothing happens
    assert!(e.tick(t0 + 10).is_empty());
    // tick past deadline (latency.max(20)): ACK fires
    let events = e.tick(t0 + 100);
    assert!(events.iter().any(|ev| matches!(
        ev,
        SyncEvent::MessageStatusChanged { status: SyncStatus::Synced, .. }
    )));
    // non-bot conversation → no scheduled reply
    let stored = e.storage().peek_messages_by_conversation("conv_linus");
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].status, SyncStatus::Synced);
}

#[test]
fn bot_conversation_schedules_reply() {
    let mut e = SyncEngine::new();
    let t0 = e.now();
    e.send_message(text_msg("", "conv_rust_bot", "ping").build(), t0, 1.0);
    e.tick(t0 + 100); // ACK (delivery deadline latency.max(20))
    let events = e.tick(t0 + 100 + 700 + 600); // reply window fully elapsed
    assert!(events
        .iter()
        .any(|ev| matches!(ev, SyncEvent::IncomingMessage(m)
            if m.sender_id == "bot_rust_engine")));
    assert_eq!(
        e.storage()
            .peek_messages_by_conversation("conv_rust_bot")
            .len(),
        2
    );
}

#[test]
fn flush_outbox_syncs_pending_when_back_online() {
    let mut e = offline_engine();
    e.send_message(text_msg("", "conv_rust_bot", "queued").build(), 1000, 1.0);
    assert_eq!(e.outbox().len(), 1);

    let events = e.set_network_condition(PartialNetwork {
        mode: Some(NetworkMode::Optimal),
        ..Default::default()
    });
    // flush is automatic on restore
    assert!(events.iter().any(|ev| matches!(
        ev,
        wechat_fui_native::sync::SyncEvent::MessageStatusChanged { status: SyncStatus::Synced, .. }
    )));
    assert!(e.outbox().is_empty());
}

#[test]
fn heartbeat_skipped_offline() {
    let mut e = offline_engine();
    assert!(e.heartbeat().is_none());
    let mut e2 = SyncEngine::new();
    assert!(e2.heartbeat().is_some());
}

#[test]
fn manual_sync_two_phase() {
    let mut e = SyncEngine::new();
    let (events, delay) = e.trigger_manual_sync();
    assert!(!events.is_empty());
    assert!(delay >= 50);
    let events = e.complete_manual_sync();
    assert!(events.iter().any(|ev| matches!(
        ev,
        wechat_fui_native::sync::SyncEvent::SyncCompleted { seq: 1042 }
    )));
}

#[test]
fn incoming_message_assigns_seq_and_stores() {
    let mut e = SyncEngine::new();
    let events = e.receive_incoming(IncomingMessage {
        conversation_id: "conv_rust_bot".into(),
        sender_id: "bot_rust_engine".into(),
        sender_name: "Rust Sync Engine (Bot)".into(),
        sender_avatar: String::new(),
        is_self: false,
        message_type: MessageType::Text,
        content: "delta push".into(),
    });
    assert!(events
        .iter()
        .any(|ev| matches!(ev, wechat_fui_native::sync::SyncEvent::IncomingMessage(m)
            if m.seq_id == 1043 && m.status == SyncStatus::Synced)));
}

#[test]
fn network_change_logs_and_emits() {
    let mut e = SyncEngine::new();
    let events = e.set_network_condition(PartialNetwork {
        mode: Some(NetworkMode::PacketLoss),
        packet_loss_rate: Some(0.5),
        latency_ms: Some(300.0),
        ..Default::default()
    });
    assert!(matches!(&events[0], wechat_fui_native::sync::SyncEvent::NetworkChanged(c)
        if c.mode == NetworkMode::PacketLoss && c.packet_loss_rate == 0.5));
    assert_eq!(e.network(), NetworkCondition {
        mode: NetworkMode::PacketLoss,
        latency_ms: 300.0,
        packet_loss_rate: 0.5,
        bandwidth_kbps: 50_000.0,
    });
}
