//! FX layer verification: every effect renders headlessly and the gallery
//! is reachable from the sidebar.

use fenestra::prelude::*;
use fenestra::shell::{render_app, Harness};
use wechat_fui_native::ui::{Model, Msg, Tab};

fn app_on_fx() -> Model {
    let mut m = Model::new();
    m.update(Msg::SelectTab(Tab::FxGallery));
    m
}

#[test]
fn fx_gallery_renders_all_effects() {
    let mut app = app_on_fx();
    let theme = app.theme();
    let image = render_app(&mut app, &[], (1280, 800), &theme);
    let path = std::env::temp_dir().join("wechat_fui_fx_gallery.png");
    image.save(&path).expect("save png");
}

#[test]
fn fx_gallery_exposed_in_accessibility_tree() {
    let app = app_on_fx();
    let mut h = Harness::new(app, Theme::from_accent(150.0, Mode::Dark), (1280, 800));
    assert!(h.query(&by::label("Visual FX Gallery")).is_some());
    assert!(h.query(&by::label("Aurora mesh + film grain")).is_some());
    assert!(h.query(&by::label("Shadow elevation ladder")).is_some());
    assert!(h.query(&by::label("Breathing chip")).is_some());
}

#[test]
fn aurora_and_grain_are_deterministic() {
    let app = Model::new();
    let theme = app.theme();

    // Same inputs → same pixels (the determinism contract).
    let a = wechat_fui_native::fx::test_support::aurora_background_pixels(&theme, 64, 64);
    let b = wechat_fui_native::fx::test_support::aurora_background_pixels(&theme, 64, 64);
    assert_eq!(a, b);
    assert_eq!(a.len(), 64 * 64 * 4);
}

#[test]
fn aurora_pixels_are_nonflat() {
    let app = Model::new();
    let theme = app.theme();
    let px = wechat_fui_native::fx::test_support::aurora_background_pixels(&theme, 128, 128);
    let mut min_r = 255u8; let mut max_r = 0u8;
    let mut min_b = 255u8; let mut max_b = 0u8;
    for i in (0..px.len()).step_by(4) {
        min_r = min_r.min(px[i]); max_r = max_r.max(px[i]);
        min_b = min_b.min(px[i+2]); max_b = max_b.max(px[i+2]);
    }
    println!("AURORA r range [{min_r},{max_r}] b range [{min_b},{max_b}]");
    assert!(max_r > min_r, "red channel flat — mesh not rendering");
    assert!(max_b.saturating_sub(min_b) > 8, "blue channel nearly flat");
}
