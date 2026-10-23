//! The borderless window's own title bar: it drags the window, its controls
//! stay controls, and the traffic lights are the window's close / minimize /
//! zoom.

use fenestra::prelude::*;
use fenestra::shell::Harness;
use kurbo::Point;
use wechat_fui_native::ui::Model;

/// The window's minimum size (main.rs) and a roomy one.
const SIZES: [(u32, u32); 2] = [(960, 640), (1280, 800)];

fn harness(size: (u32, u32)) -> Harness<Model> {
    Harness::new(Model::new(), Theme::from_accent(150.0, Mode::Dark), size)
}

fn button(name: &str) -> fenestra::Query {
    by::role(Semantics::Button).name(name)
}

#[test]
fn the_title_bar_drags_the_window_around_its_controls() {
    for size in SIZES {
        let h = harness(size);
        let frame = h.frame();
        let tree = frame.debug_tree();
        let bar = h.get(&by::id("title-bar")).rect;
        assert!(
            bar.x0 == 0.0 && bar.y0 == 0.0 && (bar.x1 - f64::from(size.0)).abs() < 0.5,
            "{size:?}: the title bar spans the top of the window\n{tree}"
        );

        // The gap between the brand and the status metrics, and the title.
        let brand = h.get(&by::label("Rust + Fenestra Native")).rect;
        let metrics = h.get(&by::label("status metrics")).rect;
        assert!(
            brand.x1 < metrics.x0,
            "{size:?}: brand and metrics overlap\n{tree}"
        );
        let gap = Point::new((brand.x1 + metrics.x0) / 2.0, bar.center().y);
        assert!(frame.drag_region_at(gap), "{size:?}: empty title bar drags");
        let title = h.get(&by::label("微信 WeChat")).rect.center();
        assert!(frame.drag_region_at(title), "{size:?}: the title drags");

        for name in ["Trigger sync", "Toggle theme"] {
            let at = h.get(&button(name)).rect.center();
            assert!(
                !frame.drag_region_at(at),
                "{size:?}: {name} takes its press"
            );
        }
        let body = Point::new(f64::from(size.0) / 2.0, f64::from(size.1) / 2.0);
        assert!(
            !frame.drag_region_at(body),
            "{size:?}: the window body is not a handle"
        );
    }
}

#[test]
fn the_traffic_lights_are_the_window_controls() {
    for size in SIZES {
        let h = harness(size);
        let frame = h.frame();
        let bar = h.get(&by::id("title-bar")).rect;
        for (name, control) in [
            ("Close window", WindowControl::Close),
            ("Minimize window", WindowControl::Minimize),
            ("Zoom window", WindowControl::Maximize),
        ] {
            let light = h.get(&button(name)).rect;
            assert!(
                light.y0 >= bar.y0 && light.y1 <= bar.y1,
                "{size:?}: {name} sits in the title bar"
            );
            assert_eq!(
                frame.window_control_at(light.center()),
                Some(control),
                "{size:?}: {name}"
            );
            assert!(
                !frame.drag_region_at(light.center()),
                "{size:?}: {name} does not drag"
            );
        }
    }
}
