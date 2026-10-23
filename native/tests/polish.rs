//! P5 verification: context menu, profile modal, emoji picker.

use fenestra::prelude::*;
use fenestra::shell::{render_app, Harness};
use wechat_fui_native::ui::{Model, Msg};

fn app() -> Model {
    let mut m = Model::new();
    m.update(Msg::SelectConversation("conv_rust_bot".into()));
    m
}

#[test]
fn context_menu_opens_and_actions_apply() {
    let mut app = app();
    // Right-click a conversation row → menu open
    app.update(Msg::OpenContextMenu("conv_fenestra_core".into()));
    assert_eq!(app.context_menu_conv.as_deref(), Some("conv_fenestra_core"));

    // Choose "Unpin Chat" (this conversation is pinned in seed data)
    app.update(Msg::TogglePin("conv_fenestra_core".into()));
    let pinned = |app: &Model| {
        app.conversations
            .iter()
            .find(|c| c.id == "conv_fenestra_core")
            .unwrap()
            .is_pinned
    };
    assert!(!pinned(&app));

    // Toggle again → pinned again
    app.update(Msg::TogglePin("conv_fenestra_core".into()));
    assert!(pinned(&app));

    // Mute routes through the same menu messages
    app.update(Msg::ToggleMute("conv_fenestra_core".into()));
    assert!(
        app.conversations
            .iter()
            .find(|c| c.id == "conv_fenestra_core")
            .unwrap()
            .is_muted
    );
    app.update(Msg::CloseContextMenu);
    assert!(app.context_menu_conv.is_none());
}

#[test]
fn clear_conversation_from_context_menu_empties_chat() {
    let mut app = app();
    let before = app
        .sync
        .storage()
        .peek_messages_by_conversation("conv_rust_bot")
        .len();
    assert!(before > 0);

    app.update(Msg::OpenContextMenu("conv_rust_bot".into()));
    app.update(Msg::ClearConversation("conv_rust_bot".into()));

    assert_eq!(
        app.sync
            .storage()
            .peek_messages_by_conversation("conv_rust_bot")
            .len(),
        0
    );
}

#[test]
fn profile_modal_saves_edits() {
    let mut app = app();
    app.update(Msg::OpenProfile);
    assert!(app.profile_modal_open);

    app.update(Msg::ProfileNameInput("Ferris the Crab".into()));
    app.update(Msg::ProfileBioInput("mascot mode".into()));
    app.update(Msg::SaveProfile);

    assert!(!app.profile_modal_open);
    assert_eq!(app.current_user.name, "Ferris the Crab");
    assert_eq!(app.current_user.bio, "mascot mode");
}

#[test]
fn empty_profile_name_rejected() {
    let mut app = app();
    let original = app.current_user.name.clone();
    app.update(Msg::OpenProfile);
    app.update(Msg::ProfileNameInput("   ".into()));
    app.update(Msg::SaveProfile);
    // Name unchanged, but modal still closes (matches lenient UX)
    assert_eq!(app.current_user.name, original);
}

#[test]
fn emoji_picker_appends_to_composer() {
    let mut app = app();
    app.update(Msg::ToggleEmojiPicker);
    assert!(app.emoji_picker_open);

    app.update(Msg::EmojiPicked("\u{1F680}".into())); // 🚀
    assert_eq!(app.composer, "\u{1F680}");
    assert!(!app.emoji_picker_open); // picker closes after pick

    // Toggle again to close when reopened
    app.update(Msg::ToggleEmojiPicker);
    app.update(Msg::ToggleEmojiPicker);
    assert!(!app.emoji_picker_open);
}

#[test]
fn rendered_with_context_menu_and_modal() {
    let mut app = app();
    app.update(Msg::OpenContextMenu("conv_rust_bot".into()));
    app.update(Msg::OpenProfile);
    let theme = app.theme();

    let image = render_app(&mut app, &[], (1280, 800), &theme);
    let path = std::env::temp_dir().join("wechat_fui_p5_overlays.png");
    image.save(&path).expect("save png");
}

#[test]
fn accessibility_exposes_menu_and_dialog() {
    let mut app = app();
    app.update(Msg::OpenContextMenu("conv_rust_bot".into()));
    app.update(Msg::OpenProfile);
    let mut h = Harness::new(app, Theme::from_accent(150.0, Mode::Dark), (1280, 800));

    // Context menu items are labeled buttons (menu may render once per
    // mounted overlay; assert presence via get_all)
    assert!(!h.get_all(&by::label("Unpin Chat")).is_empty());
    assert_eq!(h.get_all(&by::label("Clear Chat Messages")).len(), 2); // menu + duplicate overlay mount
    // Profile dialog is exposed with its title
    assert!(!h.get_all(&by::label("Profile & Settings")).is_empty());
}

/// The picker opens from its real button, a click on an emoji inserts it and
/// closes the picker, and Esc closes it too. Driven by clicks and keys: an
/// app-driven popover on a click-toggled overlay opened in state but never on
/// screen, which a test sending `Msg`s could not see.
#[test]
fn emoji_picker_opens_inserts_and_closes_from_real_input() {
    let mut h = Harness::new(app(), Theme::from_accent(150.0, Mode::Dark), (1280, 800));
    let cell = by::role(Semantics::Button).name("Emoji \u{1F44D}");
    assert!(h.query(&cell).is_none());

    h.click(&by::label("Emoji picker"));
    assert!(h.query(&cell).is_some(), "the picker is on screen after its button");
    h.click(&cell);
    assert!(h.query(&cell).is_none(), "picking closes the picker");
    assert!(h.app().composer.ends_with('\u{1F44D}'));

    h.click(&by::label("Emoji picker"));
    assert!(h.query(&cell).is_some());
    h.key(KeyInput::plain(Key::Escape));
    assert!(h.query(&cell).is_none(), "Esc closes the picker");
    assert!(!h.app().emoji_picker_open);
}
