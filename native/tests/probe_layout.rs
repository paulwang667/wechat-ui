//! Layout probe: prints resolved rects for the chat pane.
//! Run: cargo test --test probe_layout --ignored -- --nocapture

use fenestra::prelude::*;
use fenestra::{Fonts, FrameState, build_frame};
use wechat_fui_native::ui::{Model, Msg};

#[test]
#[ignore = "prints the whole layout tree"]
fn probe_the_chat_layout() {
    let mut app = Model::new();
    app.update(Msg::SelectConversation("conv_rust_bot".into()));
    let view = app.view();
    let mut fonts = Fonts::embedded();
    let mut state = FrameState::new();
    let frame = build_frame(&view, &app.theme(), &mut fonts, &mut state, (1280.0, 800.0), 1.0);
    for line in frame.debug_tree().lines() {
        println!("{line}");
    }
}
