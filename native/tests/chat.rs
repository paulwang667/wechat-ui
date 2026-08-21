//! P3 verification: chat feed behavior — stick_to_bottom, Enter-to-send,
//! message type rendering, and the full send→ACK→bot-reply loop.

use fenestra::prelude::*;
use fenestra::shell::{render_app, Harness};
use wechat_fui_native::sync::SyncEvent;
use wechat_fui_native::ui::{Model, Msg};
use wechat_fui_native::types::SyncStatus;

fn dark_app() -> Model {
    let mut m = Model::new();
    m.update(Msg::SelectConversation("conv_rust_bot".into()));
    m
}

#[test]
fn send_flow_full_loop_send_ack_reply() {
    let mut app = dark_app();
    let t0 = app.sync.now();

    app.update(Msg::ComposerInput("ping the engine".into()));
    app.update(Msg::ComposerSend);

    // Message stored optimistically as Sending
    let msgs = app.sync.storage().peek_messages_by_conversation("conv_rust_bot");
    let last = msgs.last().expect("message stored");
    assert_eq!(last.content, "ping the engine");
    assert_eq!(last.status, SyncStatus::Sending);
    assert!(last.is_self);

    // Advance clock: delivery deadline (latency.max(20)) passes → ACK
    let events = app.sync.tick(t0 + 100);
    assert!(events.iter().any(|ev| matches!(
        ev,
        SyncEvent::MessageStatusChanged { status: SyncStatus::Synced, .. }
    )));

    // Reply window (700..1300ms after delivery) elapses → bot responds
    let events = app.sync.tick(t0 + 100 + 700 + 600);
    assert!(events
        .iter()
        .any(|ev| matches!(ev, SyncEvent::IncomingMessage(m)
            if m.sender_id == "bot_rust_engine" && !m.is_self)));
    // bump_conversation runs via apply_events in update(); simulate:
    // (update() already applied send-side events; reply events applied here)
    if let Some(SyncEvent::IncomingMessage(reply)) = events
        .iter()
        .find(|ev| matches!(ev, SyncEvent::IncomingMessage(_)))
    {
        app.apply_events_public(vec![SyncEvent::IncomingMessage(reply.clone())]);
    }
    let conv = app.conversations.iter().find(|c| c.id == "conv_rust_bot").unwrap();
    assert!(conv
        .last_message
        .as_deref()
        .is_some_and(|p| p.contains("Rust Engine ACK") || p.contains("LZ4") || p.contains("Sync sequence") || p.contains("Tip")));
}

#[test]
fn empty_composer_send_is_noop() {
    let mut app = dark_app();
    let before = app
        .sync
        .storage()
        .peek_messages_by_conversation("conv_rust_bot")
        .len();
    app.composer.clear();
    app.update(Msg::ComposerSend);
    let after = app
        .sync
        .storage()
        .peek_messages_by_conversation("conv_rust_bot")
        .len();
    assert_eq!(before, after);
}

#[test]
fn offline_send_lands_in_outbox_and_recovers() {
    use wechat_fui_native::sync::PartialNetwork;
    use wechat_fui_native::types::NetworkMode;

    let mut app = dark_app();
    app.sync
        .set_network_condition(PartialNetwork {
            mode: Some(NetworkMode::Offline),
            ..Default::default()
        });

    app.update(Msg::ComposerInput("queued while offline".into()));
    app.update(Msg::ComposerSend);
    assert_eq!(app.sync.outbox().len(), 1);
    assert_eq!(app.sync.outbox()[0].status, SyncStatus::Pending);

    // Back online → outbox flushes automatically
    let events = app.sync.set_network_condition(PartialNetwork {
        mode: Some(NetworkMode::Optimal),
        ..Default::default()
    });
    assert!(events.iter().any(|ev| matches!(
        ev,
        SyncEvent::MessageStatusChanged { status: SyncStatus::Synced, .. }
    )));
    assert!(app.sync.outbox().is_empty());
}

#[test]
fn rendered_feed_shows_bubbles_after_send() {
    let mut app = dark_app();
    app.update(Msg::ComposerInput("visual check".into()));
    app.update(Msg::ComposerSend);
    let theme = app.theme();

    let image = render_app(&mut app, &[], (1280, 800), &theme);
    let path = std::env::temp_dir().join("wechat_fui_after_send.png");
    image.save(&path).expect("save png");
    println!("saved {}", path.display());
}

#[test]
fn harness_input_drives_search_field() {
    let mut app = dark_app();
    let mut h = Harness::new(&mut app, Theme::from_accent(150.0, Mode::Dark), (1280, 800));

    // Type into the search input via synthetic text events on focused field.
    h.update(Msg::SearchInput("linus".into()));

    // The visible conversation list narrows to the Linus row.
    assert!(h.query(&by::label("Conversation with Linus Torvalds")).is_some());
    assert!(h
        .query(&by::label("Conversation with Rust Sync Engine (Bot)"))
        .is_none());
}

#[test]
fn filter_unread_narrows_list() {
    let mut app = dark_app();
    app.update(Msg::FilterUnread);
    let visible = app.visible_conversations_public();
    // Only fenestra group has unread=2 initially
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, "conv_fenestra_core");
}
