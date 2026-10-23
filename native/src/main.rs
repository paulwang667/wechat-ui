use fenestra::prelude::*;
use wechat_fui_native::ui::{Model, Msg};

fn main() {
    let mut app = Model::new();
    // Open the bot conversation by default (mirrors the TS app's initial state).
    app.update(Msg::SelectConversation("conv_rust_bot".into()));
    // Borderless: the app draws its own title bar (`ui::title_bar`), which
    // drags the window and carries working close / minimize / zoom lights.
    fenestra::run(
        app,
        WindowOptions::titled("微信 WeChat — Rust + Fenestra Native")
            .borderless()
            .with_min_size(960.0, 640.0),
    )
}
