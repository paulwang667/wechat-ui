//! Glass lab: an evaluation bench for fenestra's frosted / liquid glass.
//!
//! One view, two runners:
//! - `cargo run --release --example glass_lab` opens a live window (real
//!   two-pass blur on the fork). Pick a scene on the left, scroll the content
//!   under the glass, flip glass ↔ solid to compare, and turn on the 60 Hz
//!   tick to read the frame interval the two-pass path actually achieves.
//! - `cargo run --release --example glass_lab -- --shots glass-lab` renders
//!   every scene × light/dark × glass/solid headlessly to PNGs and prints the
//!   per-scene render time, glass vs solid, plus a pane-count sweep.
//!
//! Every color comes from theme tokens; the WeChat accent hue (150°) matches
//! the app.

use fenestra::prelude::*;
use fenestra::shell::Harness;
use std::time::{Duration, Instant};

const SIZE: (u32, u32) = (1240, 800);
const HEADER_H: f32 = 56.0;
/// Hiragino ships with macOS and covers both Han and Latin; registered as the
/// Display role so headless renders draw the Chinese copy instead of tofu.
const CJK_FONT: &str = "/System/Library/Fonts/Hiragino Sans GB.ttc";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scene {
    StickyHeader,
    FloatingPill,
    Popover,
    Modal,
    Media,
    SidebarEdge,
    Sweep,
    AntiPatterns,
    Bench(u8),
}

const SCENES: [(Scene, &str, &str); 8] = [
    (Scene::StickyHeader, "header", "吸顶条：气泡从下方滚过"),
    (Scene::FloatingPill, "pill", "浮动胶囊 / 来电卡"),
    (Scene::Popover, "popover", "弹层：右键菜单 / 表情"),
    (Scene::Modal, "modal", "模态：kit modal"),
    (Scene::Media, "media", "媒体上的控件条"),
    (Scene::SidebarEdge, "sidebar", "贴边侧栏 / 抽屉"),
    (Scene::Sweep, "sweep", "参数扫描：透明度 × 模糊"),
    (Scene::AntiPatterns, "anti", "反例：不该用玻璃的地方"),
];

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Pick(Scene),
    ToggleGlass,
    ToggleDark,
    ToggleTick,
    Tick,
    Noop,
}

struct Lab {
    scene: Scene,
    glass: bool,
    dark: bool,
    tick: bool,
    last_tick: Option<Instant>,
    /// Redraws per second over the last full second (from `views`), while
    /// continuous redraw is on — the real frame rate, unlike the tick interval,
    /// which only says how often the timer fires.
    fps: f64,
    fps_window: Option<(Instant, u32)>,
    /// `--live-bench`: the remaining (scene, glass) segments to measure.
    plan: Vec<(Scene, bool, bool)>,
    /// Bench: slide the feed under the glass every frame, so the backdrop
    /// changes like it does while scrolling.
    motion: bool,
    frame_no: u32,
    seg_start: Option<Instant>,
    /// `view()` calls: one per redraw, so per-second deltas are the real
    /// frame rate even when ticks queue up behind a slow frame.
    views: std::cell::Cell<u32>,
    views_at_warmup: Option<u32>,
    benching: bool,
}

/// Live-bench segment length; the first `WARMUP` of each is discarded.
const SEGMENT: Duration = Duration::from_secs(4);
const WARMUP: Duration = Duration::from_millis(800);

impl Lab {
    fn new(scene: Scene, glass: bool, dark: bool) -> Self {
        Self {
            scene,
            glass,
            dark,
            tick: false,
            last_tick: None,
            fps: 0.0,
            fps_window: None,
            plan: Vec::new(),
            motion: false,
            frame_no: 0,
            seg_start: None,
            views: std::cell::Cell::new(0),
            views_at_warmup: None,
            benching: false,
        }
    }

    /// Records one tick interval for the live bench; on a segment boundary
    /// prints its stats and moves to the next (scene, glass), exiting at the end.
    fn bench_step(&mut self, now: Instant) {
        let start = *self.seg_start.get_or_insert(now);
        let age = now.duration_since(start);
        if age > WARMUP {
            self.views_at_warmup.get_or_insert(self.views.get());
        }
        if age < SEGMENT {
            return;
        }
        let frames = self.views.get() - self.views_at_warmup.take().unwrap_or(0);
        let fps = f64::from(frames) / (SEGMENT - WARMUP).as_secs_f64();
        let slug = SCENES.iter().find(|(s, ..)| *s == self.scene).map_or("bench4", |(_, slug, _)| *slug);
        println!(
            "{slug:<10} {:>6} {:>7} {fps:>7.1} {:>9.1}",
            if self.glass { "glass" } else { "solid" },
            if self.motion { "moving" } else { "static" },
            1000.0 / fps,
        );
        self.seg_start = None;
        if self.plan.is_empty() {
            std::process::exit(0);
        }
        let (scene, glass, motion) = self.plan.remove(0);
        self.scene = scene;
        self.glass = glass;
        self.motion = motion;
    }
}

impl App for Lab {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Pick(s) => self.scene = s,
            Msg::ToggleGlass => self.glass = !self.glass,
            Msg::ToggleDark => self.dark = !self.dark,
            Msg::ToggleTick => {
                self.tick = !self.tick;
                self.last_tick = None;
                self.fps = 0.0;
                self.fps_window = None;
            }
            Msg::Tick => {
                self.frame_no = self.frame_no.wrapping_add(1);
                let now = Instant::now();
                if self.last_tick.is_some() && self.benching {
                    self.bench_step(now);
                }
                self.last_tick = Some(now);
                let (start, views) = *self.fps_window.get_or_insert((now, self.views.get()));
                let elapsed = now.duration_since(start).as_secs_f64();
                if elapsed >= 1.0 {
                    self.fps = f64::from(self.views.get() - views) / elapsed;
                    self.fps_window = Some((now, self.views.get()));
                }
            }
            Msg::Noop => {}
        }
    }

    fn subscriptions(&self) -> Vec<Sub<Msg>> {
        if self.tick {
            // Ticks far faster than the display so frames queue: the redraw rate
            // then measures the frame's cost instead of the timer's period.
            vec![Sub::every("lab-tick", Duration::from_millis(1), || Msg::Tick)]
        } else {
            Vec::new()
        }
    }

    fn theme(&self) -> Theme {
        Theme::from_accent(150.0, if self.dark { Mode::Dark } else { Mode::Light })
    }

    fn view(&self) -> Element<Msg> {
        self.views.set(self.views.get() + 1);
        let shift = if self.motion { (self.frame_no % 48) as f32 } else { 0.0 };
        FEED_SHIFT.with(|c| c.set(shift));
        row()
            .id("lab")
            .w_full()
            .h_full()
            .themed(|t, s| s.bg(t.bg))
            .children([controls(self), stage(self)])
    }
}

// ---------- Text ----------

fn cn(s: impl Into<String>) -> Element<Msg> {
    text(s.into()).family(FamilyRole::Display)
}

/// A bordered action chip. Kit `button` labels use the embedded Sans face,
/// which has no CJK, so the lab's Chinese buttons are built on `cn`.
fn action(label: impl Into<String>, msg: Msg, danger: bool) -> Element<Msg> {
    let label = label.into();
    row()
        .h(32.0)
        .px(SP3)
        .items_center()
        .justify_center()
        .rounded(R_MD)
        .cursor(Cursor::Pointer)
        .semantics(Semantics::Button)
        .label(label.clone())
        .transition(Transition::colors())
        .state_layer(|t| t.text)
        .themed(move |t, s| {
            if danger { s.bg(t.danger.solid) } else { s.bg(t.element).border(1.0, t.border_subtle) }
        })
        .on_click(msg)
        .child(cn(label).size(TextSize::Sm).themed(move |t, s| {
            s.color(if danger { t.text_on(t.danger.solid) } else { t.text })
        }))
}

// ---------- Controls column (solid: it is chrome, not the subject) ----------

fn controls(lab: &Lab) -> Element<Msg> {
    let scene_btn = |(scene, _, label): &(Scene, &str, &str)| -> Element<Msg> {
        let active = lab.scene == *scene;
        let scene = *scene;
        row()
            .w_full()
            .h(32.0)
            .px(SP3)
            .items_center()
            .rounded(R_SM)
            .cursor(Cursor::Pointer)
            .semantics(Semantics::Button)
            .label(*label)
            .transition(Transition::colors())
            .state_layer(|t| t.text)
            .themed(move |t, s| if active { s.bg(t.accent_bg) } else { s })
            .on_click(Msg::Pick(scene))
            .child(cn(*label).size(TextSize::Sm).themed(move |t, s| {
                s.color(if active { t.accent_text } else { t.text })
            }))
            .into()
    };
    let tick_label = if lab.tick {
        if lab.fps > 0.0 {
            format!("重绘 {:.0} fps · 每帧 {:.1} ms", lab.fps, 1000.0 / lab.fps)
        } else {
            "持续重绘：测量中…".into()
        }
    } else {
        "持续重绘：关".into()
    };
    col()
        .id("controls")
        .w(236.0)
        .shrink0()
        .h_full()
        .p(SP3)
        .gap(SP1)
        .themed(|t, s| s.bg(t.surface).border_right(1.0, t.border_subtle))
        .children([
            cn("Glass Lab").size(TextSize::Lg).weight(Weight::Semibold),
            cn("fenestra 玻璃态评估").size(TextSize::Xs).themed(|t, s| s.color(t.text_muted)),
            div().h(SP3),
        ])
        .children(SCENES.iter().map(scene_btn).collect::<Vec<_>>())
        .children([
            spacer(),
            action(if lab.glass { "玻璃：开" } else { "玻璃：关（实色对照）" }, Msg::ToggleGlass, false),
            action(if lab.dark { "暗色" } else { "亮色" }, Msg::ToggleDark, false),
            action(tick_label, Msg::ToggleTick, false),
        ])
}

// ---------- Optics: one recipe, trimmed per placement ----------

/// What a glass pane draws on top of its blurred backdrop. `Solid` is the
/// control: same geometry, an opaque token fill, no blur.
#[derive(Clone, Copy)]
struct Optics {
    fill: f32,
    blur: f32,
    vibrancy: f32,
    rim: bool,
    sheen: bool,
    adaptive: bool,
}

impl Optics {
    /// Floating pane: the full liquid recipe.
    const FLOAT: Self = Self { fill: 0.55, blur: 14.0, vibrancy: 1.5, rim: true, sheen: true, adaptive: true };
    /// Text-carrying popover: legibility first.
    const POPOVER: Self = Self { fill: 0.78, blur: 18.0, vibrancy: 1.5, rim: true, sheen: false, adaptive: true };
    /// Docked strip / edge: blur + vibrancy only; no rim, sheen or adaptive
    /// tint, so it stays level with the opaque columns next to it.
    const DOCKED: Self = Self { fill: 0.62, blur: 16.0, vibrancy: 1.6, rim: false, sheen: false, adaptive: false };
}

/// Quiet rim: the stock `SpecularEdge::glass()` (0.6 / 0.18) is loud on small
/// panes.
fn quiet_rim() -> SpecularEdge {
    SpecularEdge { intensity: 0.22, shade: 0.08, ..SpecularEdge::glass() }
}

/// Applies glass (or its solid control) to `el`. `base` picks the token the
/// tint derives from; it is a fn pointer so the themed closure stays 'static.
fn glass(el: Element<Msg>, on: bool, o: Optics, base: fn(&Theme) -> Color) -> Element<Msg> {
    if !on {
        return el.themed(move |t, s| s.bg(base(t)));
    }
    el.overflow_hidden().themed(move |t, s| {
        let mut s = s
            .bg(Material::new(o.fill, o.blur, o.vibrancy).tint(base(t)))
            .backdrop_blur(o.blur)
            .backdrop_saturate(o.vibrancy);
        if o.rim {
            s = s.specular_edge(quiet_rim());
        }
        if o.sheen {
            s = s.sheen(Sheen::glass());
        }
        if o.adaptive {
            s = s.adaptive_tint(AdaptiveTint::glass());
        }
        s
    })
}

fn surface(t: &Theme) -> Color {
    t.surface
}
fn raised(t: &Theme) -> Color {
    t.surface_raised
}
/// A fixed dark base for controls over media: always the dark theme's
/// background, whatever the app mode, so a bright photo cannot wash it out.
fn media_dark(_: &Theme) -> Color {
    Theme::from_accent(150.0, Mode::Dark).bg
}

// ---------- Backdrop content ----------

const LINES: [&str; 12] = [
    "今晚的发布会几点开始？",
    "八点，会议室 3B，记得带转接头。",
    "收到，PPT 我再过一遍。",
    "顺便把 fenestra 的玻璃效果截个图给我看看",
    "好的，刚把依赖切到 fork 了，两遍渲染在实时窗口也生效了。",
    "模糊是 CPU 做的？",
    "对，三遍盒式模糊，和半径无关，主要成本在 GPU 回读。",
    "那动画帧每一帧都要付？",
    "只要画面里有玻璃就要付，所以要量。",
    "绿色气泡从玻璃下面滚过去的时候挺好看的",
    "晚点一起评审一下适用场景吧",
    "OK，我先把实验台搭出来。",
];

fn bubble(i: usize) -> Element<Msg> {
    let me = i % 3 != 1;
    let body = cn(LINES[i % LINES.len()])
        .size(TextSize::Sm)
        .themed(move |t, s| s.color(if me { t.on_accent } else { t.text }));
    let b = div()
        .max_w(420.0)
        .px(SP3)
        .py(SP2)
        .rounded(R_XL)
        .themed(move |t, s| s.bg(if me { t.accent } else { t.element_hover }))
        .child(body);
    let r = row().w_full();
    if me { r.justify_end().child(b) } else { r.child(b) }
}

/// A photo stand-in: a vivid OKLab mesh in status + accent colors.
fn photo(t: &Theme, w: u32, h: u32, seed: u32) -> Element<Msg> {
    let c = [t.accent, t.warning.solid, t.danger.solid, t.success.solid, t.accents.step(9)];
    let k = seed as usize;
    let pts = [
        (0.0, 0.0, c[k % 5]),
        (1.0, 0.1, c[(k + 1) % 5]),
        (0.2, 1.0, c[(k + 2) % 5]),
        (0.9, 0.9, c[(k + 3) % 5]),
        (0.5, 0.5, c[(k + 4) % 5]),
    ]
    .map(|(x, y, color)| effects::MeshPoint { x, y, color });
    let key = format!("{w}x{h}#{seed}:{:?}", t.accent);
    let px = PHOTOS.with(|m| m.borrow_mut().entry(key).or_insert_with(|| effects::mesh(w, h, &pts)).clone());
    image_rgba8(w, h, px)
}

thread_local! {
    /// The bench's per-frame feed offset; a thread-local so every scene's
    /// `feed` picks it up without threading the lab through each builder.
    static FEED_SHIFT: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Mesh "photos" by (w, h, seed, theme): generated once, not every frame,
    /// so the bench measures glass rather than the lab's own gradients.
    static PHOTOS: std::cell::RefCell<std::collections::HashMap<String, Vec<u8>>> = Default::default();
}

/// The scrolling chat feed every overlay scene floats over. `top` reserves
/// room so the first bubble starts below a docked header.
fn feed(t: &Theme, top: f32, bottom: f32) -> Element<Msg> {
    let mut items: Vec<Element<Msg>> = Vec::new();
    for i in 0..30 {
        items.push(bubble(i));
        if i % 9 == 4 {
            items.push(row().w_full().justify_end().child(
                photo(t, 300, 180, i as u32).w(300.0).h(180.0).rounded(R_LG),
            ));
        }
    }
    col()
        .id("lab-feed")
        .w_full()
        .h_full()
        .scroll_y()
        .stick_to_bottom()
        .px(SP6)
        .pt(top)
        .pb(bottom)
        .gap(SP3)
        .translate(0.0, FEED_SHIFT.with(std::cell::Cell::get))
        .children(items)
}

fn caption(s: &str) -> Element<Msg> {
    cn(s).size(TextSize::Xs).themed(|t, s| s.color(t.text_muted))
}

// ---------- Stage ----------

fn stage(lab: &Lab) -> Element<Msg> {
    let t = lab.theme();
    let on = lab.glass;
    let body: Vec<Element<Msg>> = match lab.scene {
        Scene::StickyHeader => sticky_header(&t, on),
        Scene::FloatingPill => floating(&t, on),
        Scene::Popover => popovers(&t, on),
        Scene::Modal => modal_scene(&t),
        Scene::Media => media(&t, on),
        Scene::SidebarEdge => sidebar_edge(&t, on),
        Scene::Sweep => sweep(&t),
        Scene::AntiPatterns => anti(&t),
        Scene::Bench(n) => bench(&t, on, n),
    };
    col().id("stage").grow().min_w(0.0).h_full().overflow_hidden().children(body)
}

fn sticky_header(t: &Theme, on: bool) -> Vec<Element<Msg>> {
    let header = row()
        .id("glass-header")
        .absolute()
        .top(0.0)
        .left(0.0)
        .w_full()
        .h(HEADER_H)
        .px(SP4)
        .gap(SP3)
        .items_center()
        .themed(|t, s| s.border_bottom(1.0, t.border_subtle))
        .children([
            div().w(32.0).h(32.0).rounded(R_MD).themed(|t, s| s.bg(t.accent)),
            col().gap(2.0).children([
                cn("fenestra 评审群").size(TextSize::Sm).weight(Weight::Semibold),
                caption("吸顶条配方：仅模糊 + 饱和，无高光无阴影"),
            ]),
        ]);
    vec![feed(t, HEADER_H + SP4, SP6), glass(header, on, Optics::DOCKED, surface)]
}

fn floating(t: &Theme, on: bool) -> Vec<Element<Msg>> {
    let pill = row()
        .id("new-msg-pill")
        .absolute()
        .bottom(SP6)
        .right(SP6)
        .h(36.0)
        .px(SP4)
        .gap(SP2)
        .items_center()
        .rounded_full()
        .shadow(ShadowToken::Lg)
        .child(cn("↓ 3 条新消息").size(TextSize::Sm).weight(Weight::Medium));
    let call = col()
        .id("call-card")
        .absolute()
        .top(SP6)
        .right(SP6)
        .w(260.0)
        .p(SP4)
        .gap(SP3)
        .rounded(R_LG)
        .shadow(ShadowToken::Lg)
        .children([
            cn("语音通话中").size(TextSize::Sm).weight(Weight::Semibold),
            text("03:21").mono().tabular().size(TextSize::Lg),
            row().gap(SP2).children([
                action("静音", Msg::Noop, false),
                action("挂断", Msg::Noop, true),
            ]),
        ]);
    vec![
        feed(t, SP6, SP16),
        glass(pill, on, Optics::FLOAT, raised),
        glass(call, on, Optics::FLOAT, raised),
    ]
}

fn chat_rows(t: &Theme) -> Element<Msg> {
    let names = ["fenestra 评审群", "张伟", "产品-小林", "Rust 中文社区", "设计组", "王芳", "运维值班", "周报机器人"];
    col().w(320.0).shrink0().h_full().themed(|t, s| s.bg(t.surface).border_right(1.0, t.border_subtle)).children(
        names
            .iter()
            .enumerate()
            .map(|(i, n)| {
                row().h(64.0).px(SP3).gap(SP3).items_center().children([
                    photo(t, 40, 40, i as u32).w(40.0).h(40.0).rounded(R_MD),
                    col().gap(2.0).children([
                        cn(*n).size(TextSize::Sm).weight(Weight::Medium),
                        caption(LINES[i]),
                    ]),
                ])
            })
            .collect::<Vec<_>>(),
    )
}

fn popovers(t: &Theme, on: bool) -> Vec<Element<Msg>> {
    let item = |s: &str, danger: bool| {
        row().h(32.0).px(SP3).items_center().rounded(R_SM).child(
            cn(s).size(TextSize::Sm).themed(move |t, st| st.color(if danger { t.danger.text } else { t.text })),
        )
    };
    let menu = col()
        .id("context-menu")
        .absolute()
        .top(150.0)
        .left(180.0)
        .w(200.0)
        .p(SP1)
        .rounded(R_LG)
        .shadow(ShadowToken::Lg)
        .children([item("置顶聊天", false), item("标为未读", false), item("消息免打扰", false), item("删除聊天", true)]);
    let emojis = ["赞", "哈哈", "收到", "好的", "稍等", "辛苦", "OK", "+1", "在吗", "马上", "谢谢", "晚安"];
    let emoji = div()
        .id("emoji-popover")
        .absolute()
        .bottom(SP6)
        .left(360.0)
        .p(SP2)
        .rounded(R_LG)
        .shadow(ShadowToken::Lg)
        .grid_cols([fenestra::Track::Fr(1.0); 6])
        .gap(SP1)
        .children(emojis.iter().map(|e| div().h(32.0).px(SP2).rounded(R_SM).items_center().justify_center().child(cn(*e).size(TextSize::Sm))).collect::<Vec<_>>());
    vec![
        row().w_full().h_full().children([chat_rows(t), feed(t, SP6, SP6)]),
        glass(menu, on, Optics::POPOVER, raised),
        glass(emoji, on, Optics::POPOVER, raised),
    ]
}

fn modal_scene(t: &Theme) -> Vec<Element<Msg>> {
    // Title left empty: the kit renders it in the embedded (non-CJK) face.
    let m: Element<Msg> = modal("")
        .max_width(420.0)
        .id("lab-modal")
        .child(cn("资料与设置").size(TextSize::Lg).weight(Weight::Semibold))
        .child(caption("fork 的 kit modal：α0.50 · blur 20 · sheen · adaptive tint（不受左侧开关影响）"))
        .child(
            row().justify_end().gap(SP2).children([
                action("取消", Msg::Noop, false),
                action("保存", Msg::Noop, false),
            ]),
        )
        .into();
    vec![row().w_full().h_full().children([chat_rows(t), feed(t, SP6, SP6)]), m]
}

fn media(t: &Theme, on: bool) -> Vec<Element<Msg>> {
    let bar = row()
        .id("media-bar")
        .absolute()
        .bottom(SP6)
        .left(SP6)
        .right(SP6)
        .h(48.0)
        .px(SP4)
        .gap(SP3)
        .items_center()
        .rounded(R_LG)
        .children([
            text("▶").size(TextSize::Lg).themed(|_, s| s.color(Theme::from_accent(150.0, Mode::Dark).text)),
            div().grow().h(4.0).rounded_full().themed(|_, s| s.bg(Theme::from_accent(150.0, Mode::Dark).border_strong)).child(
                div().w(220.0).h(4.0).rounded_full().themed(|t, s| s.bg(t.accent)),
            ),
            text("01:24 / 04:10").mono().tabular().size(TextSize::Xs).themed(|_, s| s.color(Theme::from_accent(150.0, Mode::Dark).text)),
        ]);
    let o = Optics { adaptive: false, sheen: false, ..Optics::FLOAT };
    vec![photo(t, 640, 400, 2).w_full().h_full(), glass(bar, on, o, media_dark)]
}

fn sidebar_edge(t: &Theme, on: bool) -> Vec<Element<Msg>> {
    let side = col()
        .id("glass-sidebar")
        .absolute()
        .top(0.0)
        .left(0.0)
        .w(260.0)
        .h_full()
        .p(SP3)
        .gap(SP2)
        .themed(|t, s| s.border_right(1.0, t.border_subtle))
        .children(
            ["全部", "未读", "群聊", "公众号", "文件传输助手"]
                .iter()
                .map(|s| row().h(36.0).px(SP3).items_center().rounded(R_SM).child(cn(*s).size(TextSize::Sm)))
                .collect::<Vec<_>>(),
        );
    let content = col().w_full().h_full().pl(260.0).child(feed(t, SP6, SP6));
    vec![photo(t, 640, 400, 4).absolute().top(0.0).left(0.0).w_full().h_full().opacity(0.35), content, glass(side, on, Optics::DOCKED, surface)]
}

/// Fill × blur grid over the same busy content: where glass stops reading as
/// glass (too opaque) and where text stops reading (too clear).
fn sweep(t: &Theme) -> Vec<Element<Msg>> {
    let fills = [0.35_f32, 0.58, 0.82];
    let blurs = [6.0_f32, 12.0, 24.0];
    let mut cells = Vec::new();
    for f in fills {
        for b in blurs {
            let o = Optics { fill: f, blur: b, ..Optics::FLOAT };
            let pane = col()
                .absolute()
                .top(40.0)
                .left(40.0)
                .w(200.0)
                .p(SP3)
                .gap(SP1)
                .rounded(R_LG)
                .shadow(ShadowToken::Md)
                .children([
                    text(format!("α {f:.2} · blur {b:.0}")).mono().size(TextSize::Xs).weight(Weight::Semibold),
                    cn("正文对比度检查").size(TextSize::Sm),
                    cn("次要文字 text_muted").size(TextSize::Xs).themed(|t, s| s.color(t.text_muted)),
                ]);
            cells.push(
                div().h(240.0).rounded(R_LG).overflow_hidden().children([
                    col().w_full().h_full().p(SP2).gap(SP2).children([
                        photo(t, 300, 90, (f * 10.0) as u32 + b as u32).w_full().h(90.0).rounded(R_MD),
                        bubble(0),
                        bubble(1),
                        bubble(3),
                    ]),
                    glass(pane, true, o, raised),
                ]),
            );
        }
    }
    vec![div().w_full().h_full().p(SP4).grid_cols([fenestra::Track::Fr(1.0); 3]).gap(SP3).children(cells)]
}

fn anti(t: &Theme) -> Vec<Element<Msg>> {
    let card = |title: &str, why: &str| {
        col().p(SP3).gap(SP1).rounded(R_LG).w(260.0).children([
            cn(title).size(TextSize::Sm).weight(Weight::Semibold),
            cn(why).size(TextSize::Xs).themed(|t, s| s.color(t.text_muted)),
        ])
    };
    // 1. Nothing behind it: a flat field.
    let flat = div().h(240.0).rounded(R_LG).themed(|t, s| s.bg(t.surface)).p(SP6).child(glass(
        card("纯色背景上的玻璃", "背后没有内容可模糊，只剩半透明色块"),
        true,
        Optics::FLOAT,
        raised,
    ));
    // 2. Glass inside glass: the inner blur is dropped.
    let inner = glass(card("内层玻璃", "内层模糊被丢弃"), true, Optics::FLOAT, raised);
    let nested = div().h(240.0).rounded(R_LG).overflow_hidden().children([
        photo(t, 400, 240, 1).w_full().h_full(),
        glass(col().absolute().top(SP4).left(SP4).p(SP4).rounded(R_XL).child(inner), true, Optics::FLOAT, raised),
    ]);
    // 3. The rejected composer: glass over a whisper-volume aurora wash.
    let wash = photo(t, 400, 240, 0).absolute().top(0.0).left(0.0).w_full().h_full().opacity(0.12);
    let composer = glass(
        col().absolute().bottom(0.0).left(0.0).w_full().h(96.0).p(SP3).child(card("玻璃输入区", "极光底色 α0.12 上只读出一条发灰的带子")),
        true,
        Optics::FLOAT,
        raised,
    );
    let task = div().h(240.0).rounded(R_LG).overflow_hidden().themed(|t, s| s.bg(t.bg)).children([wash, composer]);
    // 4. Subtle text on a clear pane.
    let clear = Optics { fill: 0.3, ..Optics::FLOAT };
    let faint = div().h(240.0).rounded(R_LG).overflow_hidden().children([
        photo(t, 400, 240, 3).w_full().h_full(),
        glass(
            col().absolute().top(SP6).left(SP6).p(SP4).gap(SP1).rounded(R_LG).children([
                cn("α0.30 上的 text_subtle").size(TextSize::Sm).themed(|t, s| s.color(t.text_subtle)),
                cn("通透但读不清，APCA 不达标").size(TextSize::Xs).themed(|t, s| s.color(t.text_subtle)),
            ]),
            true,
            clear,
            raised,
        ),
    ]);
    vec![div().w_full().h_full().p(SP4).grid_cols([fenestra::Track::Fr(1.0); 2]).gap(SP3).children([flat, nested, task, faint])]
}

/// `n` floating panes over the feed: how cost scales with pane count.
fn bench(t: &Theme, on: bool, n: u8) -> Vec<Element<Msg>> {
    let mut v = vec![feed(t, SP6, SP6)];
    for i in 0..n {
        let (r, c) = (f32::from(i / 4), f32::from(i % 4));
        let pane = col()
            .absolute()
            .top(SP6 + r * 200.0)
            .left(SP6 + c * 240.0)
            .w(220.0)
            .h(160.0)
            .rounded(R_LG)
            .p(SP3)
            .child(cn(format!("pane {i}")).size(TextSize::Sm));
        v.push(glass(pane, on, Optics::FLOAT, raised));
    }
    v
}

// ---------- Runners ----------

fn register_cjk_headless() {
    let data = std::fs::read(CJK_FONT).expect("read CJK font");
    fenestra::shell::with_fonts(|f| assert!(f.register(FamilyRole::Display, data), "register CJK font"));
}

/// Mean render time in ms over `runs`, after a warm-up render saved to `png`.
/// Mean render time at `scale` (2.0 = Retina), where blur and read-back
/// work on physical pixels.
fn time_scaled(scene: Scene, glass: bool, runs: u32, scale: f64) -> f64 {
    let lab = Lab::new(scene, glass, true);
    let theme = lab.theme();
    let _ = fenestra::shell::render_element_scaled(lab.view(), &theme, SIZE, scale);
    let t0 = Instant::now();
    for _ in 0..runs {
        let _ = fenestra::shell::render_element_scaled(lab.view(), &theme, SIZE, scale);
    }
    t0.elapsed().as_secs_f64() * 1000.0 / f64::from(runs)
}

fn time_render(scene: Scene, glass: bool, dark: bool, runs: u32, png: Option<String>) -> f64 {
    let lab = Lab::new(scene, glass, dark);
    let theme = lab.theme();
    let mut h = Harness::new(lab, theme, SIZE);
    let img = h.render(); // warm-up: shaders, glyph and image caches
    if let Some(path) = png {
        img.save(path).expect("save png");
    }
    let t0 = Instant::now();
    for _ in 0..runs {
        let _ = h.render();
    }
    t0.elapsed().as_secs_f64() * 1000.0 / f64::from(runs)
}

fn shots(dir: &str) {
    register_cjk_headless();
    std::fs::create_dir_all(dir).expect("create output dir");
    const RUNS: u32 = 8;
    println!("{:<10} {:>6} {:>11} {:>11} {:>9}", "scene", "mode", "solid ms", "glass ms", "overhead");
    for (scene, slug, _) in SCENES {
        let comparable = !matches!(scene, Scene::Modal | Scene::Sweep | Scene::AntiPatterns);
        for dark in [false, true] {
            let mode = if dark { "dark" } else { "light" };
            let g = time_render(scene, true, dark, RUNS, Some(format!("{dir}/{slug}_{mode}_glass.png")));
            if comparable {
                let s = time_render(scene, false, dark, RUNS, Some(format!("{dir}/{slug}_{mode}_solid.png")));
                println!("{slug:<10} {mode:>6} {s:>11.1} {g:>11.1} {:>8.1}", g - s);
            } else {
                println!("{slug:<10} {mode:>6} {:>11} {g:>11.1} {:>9}", "-", "-");
            }
        }
    }
    println!("\nRetina 2x ({}x{} physical, dark):", SIZE.0 * 2, SIZE.1 * 2);
    println!("{:<10} {:>11} {:>11} {:>9}", "scene", "solid ms", "glass ms", "overhead");
    for (scene, slug, _) in SCENES {
        if matches!(scene, Scene::Modal | Scene::Sweep | Scene::AntiPatterns) {
            continue;
        }
        let (s, g) = (time_scaled(scene, false, RUNS, 2.0), time_scaled(scene, true, RUNS, 2.0));
        println!("{slug:<10} {s:>11.1} {g:>11.1} {:>8.1}", g - s);
    }
    let base2 = time_scaled(Scene::Bench(0), true, RUNS, 2.0);
    for n in [1u8, 4, 8] {
        let ms = time_scaled(Scene::Bench(n), true, RUNS, 2.0);
        println!("bench {n:>2} panes {ms:>6.1} ms  (+{:.1})", ms - base2);
    }
    println!("\npane-count sweep (220x160 floating panes, dark, {}x{}):", SIZE.0, SIZE.1);
    let base = time_render(Scene::Bench(0), true, true, RUNS, None);
    println!("  0 panes {base:>7.1} ms");
    for n in [1u8, 2, 4, 8, 12] {
        let ms = time_render(Scene::Bench(n), true, true, RUNS, None);
        println!("  {n:>2} panes {ms:>6.1} ms  (+{:.1})", ms - base);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--shots") {
        let dir = args.get(i + 1).map_or("glass-lab", String::as_str);
        shots(dir);
        return;
    }
    let mut lab = Lab::new(Scene::StickyHeader, true, true);
    if args.iter().any(|a| a == "--live-bench") {
        lab.tick = true;
        lab.benching = true;
        lab.plan = [Scene::StickyHeader, Scene::FloatingPill, Scene::Popover, Scene::SidebarEdge, Scene::Bench(4)]
            .iter()
            .flat_map(|s| [(*s, false, true), (*s, true, true), (*s, true, false)])
            .chain([(Scene::Modal, true, true)])
            .collect();
        let (scene, glass, motion) = lab.plan.remove(0);
        lab.scene = scene;
        lab.glass = glass;
        lab.motion = motion;
        println!("{:<10} {:>6} {:>7} {:>7} {:>9}", "scene", "mode", "bg", "fps", "ms/frame");
    }
    let font = std::fs::read(CJK_FONT).expect("read CJK font");
    fenestra::run(
        lab,
        WindowOptions::titled("Glass Lab — fenestra").with_size(1240.0, 800.0).with_font(FamilyRole::Display, font),
    )
}
