//! Visual-experience layer: GPU-accelerated paint effects, micro-motion, and
//! light-and-shadow treatments built on fenestra's primitives.
//!
//! Everything here is a pure function of (theme, state) — the same determinism
//! contract as the rest of the app, so every effect golden-locks.

use crate::ui::{Msg, Model};
use fenestra::prelude::*;

// ---------- 1. Mesh-gradient aurora background (effects::mesh + image_rgba8) ----------

/// A deterministic OKLab mesh field in theme colors: the "liquid light" wash
/// behind the chat area. Regenerated per theme flip; cached by size+mode.
pub fn aurora_background(m: &Model, width: u32, height: u32) -> Element<Msg> {
    let points = aurora_points(&m.theme());
    let pixels = effects::mesh(width, height, &points);
    image_rgba8(width, height, pixels)
        .w_full()
        .h_full()
        .absolute()
        .top(0.0)
        .left(0.0)
}

/// The working-surface variant: same aurora language at whisper volume.
/// Points hug the corners (away from message content), alpha ≤ 0.22, so the
/// feed reads as lit air rather than decoration. Stretched from a fixed
/// 480×320 field — mesh gradients scale cleanly.
pub fn aurora_wash(m: &Model) -> Element<Msg> {
    let t = m.theme();
    // Opaque points (mesh blends OKLab regardless of point alpha); the
    // whisper volume is applied at draw time via element opacity.
    let points = vec![
        effects::MeshPoint {
            x: 0.06,
            y: 0.08,
            color: t.accent,
        },
        effects::MeshPoint {
            x: 0.94,
            y: 0.12,
            color: t.success.solid,
        },
        effects::MeshPoint {
            x: 0.10,
            y: 0.94,
            color: t.accent_bg,
        },
        effects::MeshPoint {
            x: 0.92,
            y: 0.90,
            color: t.bg,
        },
    ];
    let pixels = effects::mesh(480, 320, &points);
    image_rgba8(480, 320, pixels)
        .opacity(0.22)
        .w_full()
        .h_full()
        .absolute()
        .top(0.0)
        .left(0.0)
}

fn aurora_points(t: &Theme) -> Vec<effects::MeshPoint> {
    vec![
        effects::MeshPoint {
            x: 0.15,
            y: 0.1,
            color: t.accent,
        },
        effects::MeshPoint {
            x: 0.85,
            y: 0.25,
            color: t.success.solid,
        },
        effects::MeshPoint {
            x: 0.3,
            y: 0.85,
            color: t.accent_bg,
        },
        effects::MeshPoint {
            x: 0.9,
            y: 0.9,
            color: t.bg,
        },
    ]
}

// ---------- 2. Film grain overlay (effects::grain) ----------

/// A seeded film-grain veil that breaks up banding on gradients and adds a
/// tactile texture over flat dark surfaces. Deterministic per seed.
pub fn grain_overlay(seed: u64, width: u32, height: u32) -> Element<Msg> {
    let pixels = effects::grain(width, height, seed, 0.05);
    image_rgba8(width, height, pixels)
        .w_full()
        .h_full()
        .absolute()
        .top(0.0)
        .left(0.0)
}

// ---------- 3. Frosted-glass panels (Surface::Glass / glass_panel) ----------

/// The composer as a floating frosted pane: content behind shows through with
/// a vibrancy tint and specular rim. In headless renders the two-pass path
/// realizes the real backdrop blur; live windows get tint + rim.
pub fn glass_composer(inner: Element<Msg>) -> Element<Msg> {
    col()
        .surface(Surface::Glass)
        .overflow_hidden()
        .p(SP2)
        .gap(SP1)
        .child(inner)
}

/// A glass command bar for the title strip: translucent, lensed edge.
pub fn glass_status_pill(label: String) -> Element<Msg> {
    div()
        .surface(Surface::Glass)
        .rounded_full()
        .px(SP3)
        .py(SP1)
        .child(
            text(label)
                .size(TextSize::Xs)
                .mono()
                .themed(|t, s| s.color(t.text)),
        )
}

// ---------- 4. Gradient paints (linear/radial/conic) ----------

/// Accent gradient banner behind the chat header: linear ramp across tokens.
pub fn gradient_header_bar() -> Element<Msg> {
    div()
        .h(3.0)
        .w_full()
        .shrink0()
        .themed(|t, s| {
            s.bg(linear_gradient(
                90.0,
                [
                    t.accent,
                    t.success.solid,
                    t.accent,
                ],
            ))
        })
}

/// A radial glow spot for empty states: soft accent halo behind an icon.
pub fn radial_glow() -> Element<Msg> {
    div()
        .w(120.0)
        .h(120.0)
        .rounded_full()
        .themed(|t, s| {
            s.bg(radial_gradient(
                (0.5, 0.5),
                0.9,
                [t.accent.with_alpha(0.35), t.accent.with_alpha(0.0)],
            ))
        })
}

// ---------- 5. Micro-motion: keyframe timelines ----------

/// Breathing glow on the "engine active" chip: a slow opacity pulse through
/// theme tokens. Loops forever; reduced motion pins stop 0.
pub fn breathing_chip(label: String) -> Element<Msg> {
    div()
        .px(SP3)
        .py(SP1)
        .rounded(R_LG)
        .keyframes(
            Keyframes::new(2600.0)
                .themed_stop(0.0, |t, s| s.bg(t.accent_bg).opacity(0.6))
                .themed_stop(0.5, |t, s| s.bg(t.accent_border).opacity(1.0))
                .themed_stop(1.0, |t, s| s.bg(t.accent_bg).opacity(0.6)),
        )
        .child(
            text(label)
                .size(TextSize::Xs)
                .mono()
                .weight(Weight::Semibold),
        )
}

/// Shimmering skeleton rows shown while a conversation's messages "load"
/// (pairs with the storage stress test): the kit's shimmer primitive.
pub fn loading_skeleton_rows(count: usize) -> Element<Msg> {
    col().gap(SP3).children(
        (0..count)
            .map(|i| {
                row().gap(SP3).children([
                    skeleton_circle(36.0),
                    col().gap(SP1).children([
                        skeleton(120.0 - (i % 3) as f32 * 20.0, 10.0),
                        skeleton(220.0 - (i % 4) as f32 * 30.0, 10.0),
                    ]),
                ])
            })
            .collect::<Vec<_>>(),
    )
}

// ---------- 6. Path draw-on: animated checkmark via trim ----------

/// A stroked checkmark that draws itself in (trim 0→1 driven by progress).
/// Pair with `Transition` or drive `progress` from app state.
pub fn draw_on_check(progress: f32) -> Element<Msg> {
    let mut p = kurbo::BezPath::new();
    p.move_to((5.0, 12.0));
    p.line_to((10.0, 17.0));
    p.line_to((19.0, 7.0));
    path(p, (24.0, 24.0), Some(2.0))
        .trim(progress.clamp(0.0, 1.0))
        .themed(|t, s| s.color(t.accent_text))
}

// ---------- 7. Enter/exit choreography ----------

/// Message bubbles fly in from below with decelerate easing when they first
/// appear (new sends), and fade-scale out on removal.
pub fn animated_bubble(el: Element<Msg>, stable_key: &str) -> Element<Msg> {
    el.id(stable_key)
        .enter(
            Transition::all()
                .duration_ms(MotionDuration::Base.ms())
                .easing(EASE_DECELERATE),
        )
        // Ghost exit: fades to 40% while dropping 8px and shrinking to 96%.
        .exit_to(0.4, 0.96, 0.0, 8.0)
}

// ---------- 8. Press/hover tactile feedback ----------

/// Standard tactile control: state layer + press dip + spring hover.
pub fn tactile(el: Element<Msg>) -> Element<Msg> {
    el.state_layer(|t| t.text)
        .press_scale()
        .transition(Transition::spring())
}

// ---------- 9. Light and shadow elevation ----------

/// Layered shadow ladder demo: how elevation reads against the dark bg.
pub fn elevation_ladder() -> Element<Msg> {
    row()
        .gap(SP4)
        .items_center()
        .children([
            shadow_card("Sm", ShadowToken::Sm),
            shadow_card("Md", ShadowToken::Md),
            shadow_card("Lg", ShadowToken::Lg),
            shadow_card("Xl", ShadowToken::Xl),
        ])
}

fn shadow_card(label: &str, token: ShadowToken) -> Element<Msg> {
    div()
        .w(72.0)
        .h(48.0)
        .rounded(R_MD)
        .shadow(token)
        .items_center()
        .justify_center()
        .themed(|t, s| s.bg(t.surface_raised))
        .child(text(label.to_string()).size(TextSize::Xs).mono())
}

// ---------- 10. Conic sweep ring (spinner alternative) ----------

/// A conic-gradient sweep ring: reads as a radar/progress halo. Static paint;
/// wrap in `.spin()` for rotation.
pub fn conic_ring(size: f32) -> Element<Msg> {
    div()
        .w(size)
        .h(size)
        .rounded_full()
        .spin(1800.0)
        .themed(|t, s| {
            s.bg(conic_gradient(
                (0.5, 0.5),
                [t.accent.with_alpha(0.0), t.accent],
            ))
            .border(1.0, t.border_subtle)
        })
}

// ---------- Demo composition: FX showcase panel ----------

/// A one-screen gallery wiring every effect above into the running app
/// (reachable from the sidebar in dev builds).
pub fn fx_showcase(m: &Model) -> Element<Msg> {
    col()
        .grow()
        .scroll_y()
        .id("fx-scroll")
        .p(SP6)
        .gap(SP6)
        .themed(|t, s| s.bg(t.bg))
        .children([
            text("Visual FX Gallery")
                .size(TextSize::Lg)
                .weight(Weight::Semibold),
            // Aurora + grain stack
            stack()
                .w_full()
                .h(160.0)
                .rounded(R_LG)
                .overflow_hidden()
                .child(aurora_background(m, 640, 320))
                .child(grain_overlay(42, 640, 320))
                .child(
                    col()
                        .p(SP4)
                        .gap(SP2)
                        .child(
                            text("Aurora mesh + film grain")
                                .size(TextSize::Sm)
                                .weight(Weight::Semibold),
                        )
                        .child(
                            text("OKLab mesh gradient, Bayer-dithered, token-colored")
                                .size(TextSize::Xs)
                                .themed(|t, s| s.color(t.text_muted)),
                        ),
                ),
            // Glass + gradients row
            row()
                .gap(SP4)
                .wrap()
                .children(vec![
                    card_base_fx("Frosted glass").child(glass_status_pill(
                        "glass pill • vibrancy tint".into(),
                    )),
                    card_base_fx("Gradient header").child(gradient_header_bar()),
                    card_base_fx("Radial glow")
                        .child(row().justify_center().child(radial_glow())),
                ]),
            // Motion row
            row()
                .gap(SP4)
                .wrap()
                .children(vec![
                    card_base_fx("Breathing chip").child(breathing_chip("ENGINE LIVE".into())),
                    card_base_fx("Conic sweep").child(conic_ring(48.0)),
                    card_base_fx("Draw-on check").child(draw_on_check(1.0)),
                    card_base_fx("Skeletons").child(loading_skeleton_rows(2)),
                ]),
            // Elevation row
            card_base_fx("Shadow elevation ladder").child(elevation_ladder()),
            // Animated bubble sample
            card_base_fx("Enter/exit choreography").child(animated_bubble(
                text_bubble_sample(),
                "fx-demo-bubble",
            )),
        ])
}

fn card_base_fx(title: &str) -> Element<Msg> {
    col()
        .p(SP4)
        .gap(SP3)
        .min_w(200.0)
        .grow()
        .rounded(R_LG)
        .themed(|t, s| s.bg(t.surface).border(1.0, t.border_subtle))
        .child(text(title.to_string()).size(TextSize::Xs).weight(Weight::Semibold))
}

fn text_bubble_sample() -> Element<Msg> {
    div()
        .max_w(360.0)
        .px(SP3)
        .py(SP2)
        .rounded(R_XL)
        .themed(|t, s| s.bg(t.accent))
        .child(
            text("Hover me — spring transition; send to see enter/exit.")
                .size(TextSize::Sm)
                .transition(Transition::spring())
                .themed(|t, s| s.color(t.on_accent)),
        )
}

// ---------- Test helpers ----------

/// Deterministic pixel access for golden/determinism tests.
pub mod test_support {
    use super::*;

    /// Pixel bytes of the aurora field at a given theme (for determinism tests).
    pub fn aurora_background_pixels(theme: &Theme, w: u32, h: u32) -> Vec<u8> {
        let points = aurora_points(theme);
        effects::mesh(w, h, &points)
    }
}
