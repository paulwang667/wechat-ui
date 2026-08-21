use fenestra::prelude::*;
use wechat_fui_native::ui::{Model, Msg};

fn main() {
    let mut app = Model::new();
    // Open the bot conversation by default (mirrors the TS app's initial state).
    app.update(Msg::SelectConversation("conv_rust_bot".into()));
    fenestra::run(app, WindowOptions::titled("微信 WeChat — Rust + Fenestra Native"))
}
