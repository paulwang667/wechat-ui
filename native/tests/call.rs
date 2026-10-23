//! The floating glass call card: started and ended from the real controls,
//! placed over the feed under the header, and never disturbing the composer.

use fenestra::prelude::*;
use fenestra::shell::Harness;
use wechat_fui_native::ui::{Model, Msg};

const SIZES: [(u32, u32); 2] = [(1280, 800), (980, 700)];
const CARD_LABEL: &str = "Voice call with Rust Sync Engine (Bot)";

/// The button itself, not the text node inside it that carries the same name.
fn button_named(name: &str) -> fenestra::Query {
    by::role(Semantics::Button).name(name)
}

fn harness(size: (u32, u32)) -> Harness<Model> {
    let mut app = Model::new();
    app.update(Msg::SelectConversation("conv_rust_bot".into()));
    Harness::new(app, Theme::from_accent(150.0, Mode::Dark), size)
}

/// The card floats inside the chat area: below the header, clear of the
/// composer, inset from the right edge, inside the window — at both sizes.
#[test]
fn call_card_floats_over_the_feed_under_the_header() {
    for size in SIZES {
        let mut h = harness(size);
        h.click(&by::label("Audio call"));
        let area = h.get(&by::id("chat-area")).rect;
        let header = h.get(&by::id("chat-header")).rect;
        let composer = h.get(&by::id("composer")).rect;
        let card = h.get(&by::id("call-card")).rect;
        let tree = h.frame().debug_tree();
        assert!(
            card.y0 >= header.y1 + 8.0,
            "{size:?}: card under the header\n{tree}"
        );
        assert!(
            card.y1 <= composer.y0 - 8.0,
            "{size:?}: card clear of the composer\n{tree}"
        );
        assert!(
            (area.x1 - card.x1 - 16.0).abs() < 0.5,
            "{size:?}: card inset 16px from the chat area's right edge\n{tree}"
        );
        assert!(
            card.x0 >= area.x0,
            "{size:?}: card inside the chat area\n{tree}"
        );
        assert!(
            card.x1 <= f64::from(size.0) && card.y1 <= f64::from(size.1),
            "{size:?}: card inside the window\n{tree}"
        );
    }
}

/// Start from the header button, mute, watch the timer, hang up — all by
/// clicking the real controls.
#[test]
fn a_call_starts_mutes_counts_and_hangs_up_from_its_controls() {
    let mut h = harness((1280, 800));
    assert!(h.query(&by::id("call-card")).is_none());

    h.click(&by::label("Audio call"));
    assert!(
        h.query(&by::label(CARD_LABEL)).is_some(),
        "the card names the call"
    );
    assert!(h.query(&by::label("00:00")).is_some());

    h.click(&button_named("Mute"));
    assert!(h.query(&button_named("Unmute")).is_some(), "mute toggles");

    for _ in 0..65 {
        h.update(Msg::CallTick);
    }
    assert!(
        h.query(&by::label("01:05")).is_some(),
        "the timer reads m:ss"
    );

    h.click(&button_named("Hang up"));
    assert!(
        h.query(&by::id("call-card")).is_none(),
        "hanging up removes the card"
    );
    assert!(h.app().call.is_none());
}

/// The card comes and goes as the last sibling of the chat area, so the
/// composer keeps its identity — and the draft in it.
#[test]
fn the_composer_keeps_its_identity_and_draft_across_a_call() {
    let mut h = harness((1280, 800));
    let before = h.get(&by::id("composer")).id;
    h.focus(&by::id("composer"));
    h.type_text("draft");

    h.click(&by::label("Video call"));
    assert!(
        h.query(&by::label("Video call with Rust Sync Engine (Bot)"))
            .is_some()
    );
    let during = h.get(&by::id("composer"));
    assert_eq!(during.id, before, "starting a call re-keyed the composer");
    assert_eq!(during.value.as_deref(), Some("draft"));

    h.click(&button_named("Hang up"));
    let after = h.get(&by::id("composer"));
    assert_eq!(after.id, before, "ending a call re-keyed the composer");
    assert_eq!(after.value.as_deref(), Some("draft"));
}
