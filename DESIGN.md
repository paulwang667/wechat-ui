# Design System — wechat-fui native

Visual contract for the fenestra-native WeChat client. Every value below is
encoded in Rust (`native/src/ui.rs`, `native/src/fx.rs`) through fenestra's
token system; this document is the human-readable mirror.

## Theme

**Dark-first.** Scene sentence: a developer evaluating a GUI stack opens the
app at night, on an OLED/Retina panel, in a dim room; the aurora wash and
glass depth only read against a dark field. Light mode exists as a
first-class flip (`Theme::from_accent(150°, Mode::Light)`) but dark is the
reference render.

- Base: `Theme::from_accent(150°, Mode::Dark)` — OKLCH-generated 12-step
  neutral ramp tinted toward hue 150 (WeChat green's family), never pure gray.
- All color flows from theme tokens (`t.accent`, `t.surface`, `t.success…`).
  Raw hex is unreachable by API design.

## Color

Strategy: **Committed-leaning product**. One saturated accent (OKLCH hue 150,
WeChat green) carries the identity: self-message bubbles, active nav, primary
buttons, focus rings, aurora wash. Neutrals are green-tinted darks.

| Role | Token | Use |
|---|---|---|
| Accent | `t.accent` / `_hover` / `_active` | Self bubbles, active states, CTA |
| Accent text | `t.accent_text` | Brand marks, code language labels |
| On-accent | `t.on_accent` | Text on green fills |
| Background | `t.bg` | App field |
| Surface | `t.surface` / `t.surface_raised` | Panels, cards, composer |
| Element | `t.element` / `_hover` / `_active` | Ghost controls, peer bubbles |
| Status | `t.success` / `t.warning` / `t.danger` | Sync states, network pills, badges |

Signature light effects (`fx.rs`):

- **Aurora mesh**: 4-point OKLab mesh (accent α0.55 → success α0.35 → bg),
  Bayer-dithered, behind chat/empty states.
- **Film grain**: seed-fixed monochrome veil at α0.05 over gradients.
- **Glass**: only for what floats over the work area, never for surfaces in
  the layout (the composer tried it and read as a grey band). One helper,
  `fx::frosted(el, Glass::…)`: a vibrancy tint of `surface_raised` over a
  real backdrop blur (saturation 1.5), quiet specular rim, adaptive tint; the
  element keeps its surface role's radius, border and shadow.
  - `Glass::POPOVER` (fill α0.78, blur 18): context menu, emoji picker —
    text over anything, legibility first.
  - `Glass::CARD` (fill α0.58, blur 14, sheen): the floating call card, so
    the feed reads through.
  - Kit modal: the fork's own glass (α0.50, blur 20), left as is.
  - Glass stays still: no translate/scale animation on a pane (the blur
    samples the untransformed rect), no glass inside glass.

## Typography

Embedded Inter (400/500/600) + JetBrains Mono for data. fenestra type scale:

| Level | Token | Use |
|---|---|---|
| Display | `TextSize::Lg` | View titles |
| Body | `TextSize::Sm` | Messages, list names |
| Caption | `TextSize::Xs` | Metadata, timestamps, seq numbers |
| Mono+tabular | `.mono().tabular()` | Seq IDs, RAM/FPS, byte sizes, logs |

Hierarchy is carried by weight (Semibold vs Regular) and color step
(`t.text` → `t.text_muted` → `t.text_subtle` → `t.text_disabled`), not size
explosion. Chat metadata lines are always Xs mono muted.

## Spacing & Layout

4px grid via `SP0..SP16`. Key rhythms:

- Three columns: 64px nav rail / 288px chat list / flex chat area.
- Chat rows: `SP3` horizontal, `SP2` vertical padding, `SP3` avatar gap.
- Feed: `SP4` padding and inter-bubble gap; bubbles max-w 520–600px.
- Cards/views: `SP6` page padding, `SP4` card padding, `SP6` section gaps.
- Radii: `R_SM`(6) chips · `R_MD`(10) avatars/inputs · `R_LG`(14) cards/code ·
  `R_XL`(20) message bubbles · `R_FULL` pills/badges.

## Elevation & Light

Layered token shadows (`ShadowToken::Sm→Xl`); dark mode leans on surface
steps more than shadow. Glass surfaces add specular edge rim + top highlight
automatically. Aurora/grain are paint, not shadow.

## Motion

MotionDuration scale (Micro 100 / Fast 120 / Base 200 / Slow 300ms),
Material easing families:

- State changes: `EASE_STANDARD`
- Entrances: `EASE_DECELERATE` (bubbles fly up-in)
- Exits: ghost `exit_to(0.4, 0.96, 0, 8)` fade-drop
- Springs: stiffness 380 / damping 26 (hover retarget)
- Ambient loops: breathing chip 2600ms opacity pulse; conic ring spin 1800ms;
  skeleton shimmer sweep. All pin to first stop under reduced motion.

## Components

Chat bubble (self: accent fill, R_XL, on-accent text · peer: element_hover,
t.text), conversation row (avatar-initials square + unread badge dot),
metric card (label/value/footnote column, no border-box sameness), network
pill (status-tinted capsule), code card (bg field + mono success-text),
context menu / modal / toast (kit surfaces), FX gallery primitives
(aurora, grain, glass pill, radial glow, draw-on check, elevation ladder).

## Accessibility

APCA contrast floors enforced by `Theme::validate_contrast`; reduced-motion
snaps all keyframes/transitions; full keyboard focus cycle with shadcn-style
focus rings; semantic tree exposes every control (verified headlessly).
