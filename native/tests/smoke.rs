//! P0 smoke: the app builds, updates, and renders headlessly.

use fenestra::prelude::*;
use fenestra::shell::render_app;
use wechat_fui_native::ui::{Model, Msg};

#[test]
fn renders_counter_headlessly() {
    let mut app = Model::new();
    let theme = app.theme();
    let image = render_app(&mut app, &[], (480, 320), &theme);
    let path = std::env::temp_dir().join("wechat_fui_smoke.png");
    image.save(&path).expect("save png");
    assert!(path.exists());
}

#[test]
fn update_changes_state() {
    use wechat_fui_native::ui::Tab;
    let mut app = Model::new();
    app.update(Msg::SelectTab(Tab::Contacts));
    assert_eq!(app.active_tab, Tab::Contacts);
}
