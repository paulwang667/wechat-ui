//! WeChat-fui native: three-column desktop shell in fenestra.
//! Ported from `src/App.tsx` + `WindowFrame` + `Sidebar` + `ChatList`.

use crate::mock;
use crate::sync::{PartialNetwork, SyncEngine, SyncEvent};
use crate::types::*;
use fenestra::prelude::*;

/// Which primary tab is active (mirrors TS `TabType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Chats,
    Contacts,
    Storage,
    SyncDiag,
    RustArch,
    Benchmarks,
    FxGallery,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Tab::Chats => "Chats",
            Tab::Contacts => "Contacts",
            Tab::Storage => "Storage & Compression",
            Tab::SyncDiag => "Sync Protocol Engine",
            Tab::RustArch => "Rust + Fenestra Blueprint",
            Tab::Benchmarks => "Performance Benchmarks",
            Tab::FxGallery => "Visual FX Gallery",
        }
    }

    /// Lucide icon name for the nav rail.
    fn icon(self) -> &'static str {
        match self {
            Tab::Chats => "message-square",
            Tab::Contacts => "users",
            Tab::Storage => "database",
            Tab::SyncDiag => "activity",
            Tab::RustArch => "code",
            Tab::Benchmarks => "gauge",
            Tab::FxGallery => "star",
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    SelectTab(Tab),
    SelectConversation(String),
    TogglePin(String),
    ToggleMute(String),
    ClearConversation(String),
    ComposerInput(String),
    ComposerSend,
    SearchInput(String),
    FilterAll,
    FilterUnread,
    FilterPinned,
    ToggleTheme,
    ManualSync,
    /// Fired by the heartbeat subscription: advances the virtual clock.
    Heartbeat,
    // P4 view actions
    VacuumCompact,
    SetStressCount(usize),
    RunStressTest(usize),
    ExportDb,
    ImportJsonInput(String),
    ImportDb,
    FlushOutbox,
    SetNetwork(NetworkMode, f64, f64),
    SetLogFilter(String),
    BenchQueryInput(String),
    RunSearchTest,
    SelectArchSnippet(usize),
    StartChatWithContact(String),
    // P5 polish
    OpenContextMenu(String),
    CloseContextMenu,
    OpenProfile,
    CloseProfile,
    ProfileNameInput(String),
    ProfileBioInput(String),
    SaveProfile,
    ToggleEmojiPicker,
    EmojiPicked(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Unread,
    Pinned,
}

pub struct Model {
    pub sync: SyncEngine,
    pub current_user: Contact,
    pub contacts: Vec<Contact>,
    pub conversations: Vec<Conversation>,
    pub active_conv_id: Option<String>,
    pub active_tab: Tab,
    pub dark_mode: bool,
    pub composer: String,
    pub search: String,
    pub filter: Filter,
    /// Bumped whenever storage changes so the view re-reads messages.
    pub messages_version: u64,
    /// Pending manual-sync completion deadline (virtual clock ms).
    manual_sync_deadline: Option<i64>,
    // P4 view state
    pub compaction_result: Option<crate::storage::CompactionReport>,
    pub stress_count: usize,
    pub is_busy: bool,
    pub import_json: String,
    pub import_status: Option<String>,
    pub log_filter: String,
    pub bench_query: String,
    pub search_test_result: Option<(usize, f64)>,
    pub arch_snippet_idx: usize,
    /// Benchmark suite results (benchmark view).
    pub benchmarks: Benchmarks,
    // P5 state
    pub context_menu_conv: Option<String>,
    pub profile_modal_open: bool,
    pub profile_name: String,
    pub profile_bio: String,
    pub emoji_picker_open: bool,
    /// Highest seq present at seed time; bubbles beyond this get enter/exit
    /// choreography (motion conveys state change, not page load).
    pub initial_max_seq: i64,
}

/// Benchmark card values (PerformanceBenchmarkView.tsx state).
#[derive(Debug, Clone)]
pub struct Benchmarks {
    pub fts_search_time_ms: f64,
    pub insert_throughput_msgs_per_sec: u64,
    pub base_memory_mb: f64,
    pub wal_checkpoint_ms: f64,
}

impl Model {
    pub fn new() -> Self {
        let now = mock::now_ms();
        let mut sync = SyncEngine::new();
        let mut max_seq = 0i64;
        for msg in mock::initial_messages(now) {
            max_seq = max_seq.max(msg.seq_id);
            sync.storage_mut().insert_message(msg);
        }
        Self {
            conversations: mock::initial_conversations(now),
            current_user: mock::current_user(),
            contacts: mock::initial_contacts(),
            sync,
            active_conv_id: Some("conv_rust_bot".into()),
            active_tab: Tab::Chats,
            dark_mode: true,
            composer: String::new(),
            search: String::new(),
            filter: Filter::All,
            messages_version: 1,
            manual_sync_deadline: None,
            compaction_result: None,
            stress_count: 2000,
            is_busy: false,
            import_json: String::new(),
            import_status: None,
            log_filter: "ALL".into(),
            bench_query: "Rust".into(),
            search_test_result: None,
            arch_snippet_idx: 0,
            benchmarks: crate::ui::Benchmarks {
                fts_search_time_ms: 0.18,
                insert_throughput_msgs_per_sec: 42_800,
                base_memory_mb: 14.8,
                wal_checkpoint_ms: 0.85,
            },
            initial_max_seq: max_seq,
            context_menu_conv: None,
            profile_modal_open: false,
            profile_name: String::new(),
            profile_bio: String::new(),
            emoji_picker_open: false,
        }
    }

    fn total_unread(&self) -> u32 {
        self.conversations.iter().map(|c| c.unread_count).sum()
    }

    fn outbox_count(&self) -> usize {
        self.sync.outbox().len()
    }

    /// Apply engine events to conversation list state. Mirrors App.tsx's
    /// syncEngine.subscribe handler.
    fn apply_events(&mut self, events: Vec<SyncEvent>) {
        for ev in events {
            match ev {
                SyncEvent::MessageSaved(msg) | SyncEvent::IncomingMessage(msg) => {
                    self.bump_conversation(&msg);
                    self.messages_version += 1;
                }
                SyncEvent::MessageStatusChanged { .. } => {
                    self.messages_version += 1;
                }
                _ => {}
            }
        }
    }

    fn bump_conversation(&mut self, msg: &Message) {
        if let Some(conv) = self
            .conversations
            .iter_mut()
            .find(|c| c.id == msg.conversation_id)
        {
            let is_unread =
                !msg.is_self && self.active_conv_id.as_deref() != Some(&msg.conversation_id);
            conv.last_message = Some(match msg.message_type {
                MessageType::Code => "[Code Snippet]".into(),
                MessageType::Voice => "[Voice Note]".into(),
                _ => msg.content.clone(),
            });
            conv.last_message_time = msg.timestamp;
            conv.last_sync_seq = msg.seq_id;
            if is_unread {
                conv.unread_count += 1;
            }
        }
    }

    /// Public passthrough for tests and future views.
    pub fn apply_events_public(&mut self, events: Vec<SyncEvent>) {
        self.apply_events(events);
    }

    /// Public passthrough for tests.
    pub fn visible_conversations_public(&self) -> Vec<&Conversation> {
        self.visible_conversations()
    }

    /// Visible conversations: filtered then pinned-first, time-descending.
    fn visible_conversations(&self) -> Vec<&Conversation> {
        let search_lc = self.search.to_lowercase();
        let mut list: Vec<&Conversation> = self
            .conversations
            .iter()
            .filter(|conv| match self.filter {
                Filter::Unread => conv.unread_count > 0,
                Filter::Pinned => conv.is_pinned,
                Filter::All => true,
            })
            .filter(|conv| {
                search_lc.is_empty()
                    || conv.name.to_lowercase().contains(&search_lc)
                    || conv
                        .last_message
                        .as_deref()
                        .is_some_and(|m| m.to_lowercase().contains(&search_lc))
            })
            .collect();
        list.sort_by(|a, b| {
            b.is_pinned
                .cmp(&a.is_pinned)
                .then(b.last_message_time.cmp(&a.last_message_time))
        });
        list
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl App for Model {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::SelectTab(tab) => self.active_tab = tab,
            Msg::SelectConversation(id) => {
                self.active_conv_id = Some(id.clone());
                self.active_tab = Tab::Chats;
                if let Some(conv) = self.conversations.iter_mut().find(|c| c.id == id) {
                    conv.unread_count = 0;
                }
            }
            Msg::TogglePin(id) => {
                if let Some(conv) = self.conversations.iter_mut().find(|c| c.id == id) {
                    conv.is_pinned = !conv.is_pinned;
                }
            }
            Msg::ToggleMute(id) => {
                if let Some(conv) = self.conversations.iter_mut().find(|c| c.id == id) {
                    conv.is_muted = !conv.is_muted;
                }
            }
            Msg::ClearConversation(id) => {
                self.sync.storage_mut().clear_conversation(&id);
                if let Some(conv) = self.conversations.iter_mut().find(|c| c.id == id) {
                    conv.last_message = None;
                    conv.unread_count = 0;
                }
                self.messages_version += 1;
            }
            Msg::ComposerInput(s) => self.composer = s,
            Msg::ComposerSend => {
                let content = self.composer.trim().to_string();
                if content.is_empty() {
                    return;
                }
                let Some(conv_id) = self.active_conv_id.clone() else {
                    return;
                };
                self.composer.clear();
                let user = &self.current_user;
                let outgoing = Message {
                    id: String::new(), // engine assigns when empty
                    conversation_id: conv_id,
                    sender_id: user.id.clone(),
                    sender_name: user.name.clone(),
                    sender_avatar: user.avatar.clone(),
                    is_self: true,
                    message_type: MessageType::Text,
                    content,
                    timestamp: self.sync.now(),
                    seq_id: 0,
                    status: SyncStatus::Sending,
                    media_url: None,
                    file_name: None,
                    file_size: None,
                    voice_duration: None,
                    code_language: None,
                    code_snippet: None,
                    quoted_message: None,
                    raw_bytes: None,
                    compressed_bytes: None,
                    is_compressed: false,
                };
                let events = self.sync.send_message(outgoing, self.sync.now(), 1.0);
                self.apply_events(events);
            }
            Msg::SearchInput(s) => self.search = s,
            Msg::FilterAll => self.filter = Filter::All,
            Msg::FilterUnread => self.filter = Filter::Unread,
            Msg::FilterPinned => self.filter = Filter::Pinned,
            Msg::ToggleTheme => self.dark_mode = !self.dark_mode,
            Msg::ManualSync => {
                let (events, delay) = self.sync.trigger_manual_sync();
                self.apply_events(events);
                self.manual_sync_deadline = Some(self.sync.now() + delay);
            }
            Msg::VacuumCompact => {
                let report = self.sync.storage_mut().vacuum_and_compact();
                self.compaction_result = Some(report);
            }
            Msg::SetStressCount(n) => self.stress_count = n,
            Msg::RunStressTest(n) => {
                self.is_busy = true;
                let conv_id = self.active_conv_id.clone().unwrap_or_else(|| "conv_rust_bot".into());
                let result = self.sync.storage_mut().batch_insert_stress_test(n, &conv_id);
                self.is_busy = false;
                self.messages_version += 1;
                let _ = result;
            }
            Msg::ExportDb => {
                let json = self.sync.storage().export_database_json();
                // Write next to the app for the demo; a real build would use rfd.
                let path = std::env::temp_dir().join("wechat_fui_db_backup.json");
                if std::fs::write(&path, json).is_ok() {
                    self.import_status = Some(format!("Exported to {}", path.display()));
                } else {
                    self.import_status = Some("Export failed".into());
                }
            }
            Msg::ImportJsonInput(s) => self.import_json = s,
            Msg::ImportDb => {
                match self.sync.storage_mut().import_database_json(&self.import_json.clone()) {
                    Ok(count) => {
                        self.import_status = Some(format!("Successfully restored {count} messages into Rust storage!"));
                        self.import_json.clear();
                        self.messages_version += 1;
                    }
                    Err(e) => self.import_status = Some(format!("Import error: {e}")),
                }
            }
            Msg::FlushOutbox => {
                let events = self.sync.flush_outbox();
                self.apply_events(events);
            }
            Msg::SetNetwork(mode, latency, loss) => {
                let events = self.sync.set_network_condition(PartialNetwork {
                    mode: Some(mode),
                    latency_ms: Some(latency),
                    packet_loss_rate: Some(loss),
                    ..Default::default()
                });
                self.apply_events(events);
            }
            Msg::SetLogFilter(f) => self.log_filter = f,
            Msg::BenchQueryInput(q) => self.bench_query = q,
            Msg::RunSearchTest => {
                let (results, ms) = self.sync.storage_mut().search_messages(&self.bench_query.clone(), None);
                self.search_test_result = Some((results.len(), ms));
            }
            Msg::SelectArchSnippet(i) => self.arch_snippet_idx = i,
            Msg::OpenContextMenu(conv_id) => self.context_menu_conv = Some(conv_id),
            Msg::CloseContextMenu => self.context_menu_conv = None,
            Msg::OpenProfile => {
                self.profile_name = self.current_user.name.clone();
                self.profile_bio = self.current_user.bio.clone();
                self.profile_modal_open = true;
            }
            Msg::CloseProfile => self.profile_modal_open = false,
            Msg::ProfileNameInput(s) => self.profile_name = s,
            Msg::ProfileBioInput(s) => self.profile_bio = s,
            Msg::SaveProfile => {
                if !self.profile_name.trim().is_empty() {
                    self.current_user.name = self.profile_name.trim().to_string();
                }
                self.current_user.bio = self.profile_bio.clone();
                self.profile_modal_open = false;
            }
            Msg::ToggleEmojiPicker => self.emoji_picker_open = !self.emoji_picker_open,
            Msg::EmojiPicked(e) => {
                self.composer.push_str(&e);
                self.emoji_picker_open = false;
            }
            Msg::StartChatWithContact(contact_id) => {
                // Find or create conversation, then switch to chats.
                let conv_id = format!("conv_{contact_id}");
                if !self.conversations.iter().any(|c| c.id == conv_id) {
                    if let Some(contact) = self.contacts.iter().find(|c| c.id == contact_id) {
                        let contact = contact.clone();
                        self.conversations.insert(
                            0,
                            Conversation {
                                id: conv_id.clone(),
                                name: contact.name.clone(),
                                avatar: contact.avatar.clone(),
                                is_group: false,
                                group_members: vec![],
                                last_message: Some(format!("Started chat with {}", contact.name)),
                                last_message_time: self.sync.now(),
                                unread_count: 0,
                                is_pinned: false,
                                is_muted: false,
                                draft: None,
                                last_sync_seq: self.sync.current_seq(),
                            },
                        );
                    }
                }
                self.active_conv_id = Some(conv_id);
                self.active_tab = Tab::Chats;
            }
            Msg::Heartbeat => {
                // Advance virtual clock; fire due deliveries.
                let now = self.sync.now();
                let events = self.sync.tick(now);
                self.apply_events(events);
                if let Some(ev) = self.sync.heartbeat() {
                    self.apply_events(vec![ev]);
                }
                // Manual-sync completion check
                if let Some(deadline) = self.manual_sync_deadline {
                    if now >= deadline {
                        let events = self.sync.complete_manual_sync();
                        self.apply_events(events);
                        self.manual_sync_deadline = None;
                    }
                }
            }
        }
    }

    fn subscriptions(&self) -> Vec<Sub<Msg>> {
        vec![Sub::every(
            "heartbeat",
            std::time::Duration::from_secs(3),
            || Msg::Heartbeat,
        )]
    }

    fn theme(&self) -> Theme {
        // WeChat green accent (~150° OKLCH); mirrors the reference shots.
        Theme::from_accent(
            150.0,
            if self.dark_mode {
                Mode::Dark
            } else {
                Mode::Light
            },
        )
    }

    fn view(&self) -> Element<Msg> {
        let mut root = col()
            .w_full()
            .h_full()
            .overflow_hidden()
            .themed(|t, s| s.bg(t.bg))
            .children([
                title_bar(self),
                crate::fx::gradient_header_bar(),
                main_row(self),
            ]);
        if self.profile_modal_open {
            root = root.child(profile_modal(self));
        }
        root
    }
}

// ---------- Title bar (WindowFrame.tsx) ----------

fn title_bar(m: &Model) -> Element<Msg> {
    let net = m.sync.network();

    row()
        .h(36.0)
        .px(SP3)
        .items_center()
        .justify_between()
        .shrink0()
        .themed(|t, s| s.bg(t.surface).border_bottom(1.0, t.border_subtle))
        .children([
            // Left: traffic lights + brand
            row()
                .gap(SP2)
                .items_center()
                .children([
                    traffic_light(0xFF5F56, 0xE0443E),
                    traffic_light(0xFFBD2E, 0xDEA123),
                    traffic_light(0x27C93F, 0x1AAB29),
                    spacer_w(12.0),
                    text("微信 WeChat")
                        .size(TextSize::Xs)
                        .weight(Weight::Semibold)
                        .themed(|t, s| s.color(t.accent_text)),
                    pill_text("Rust + Fenestra Native"),
                ])
                .shrink0(),
            // Center: RAM / FPS / Seq
            row()
                .gap(SP4)
                .items_center()
                .label("status metrics")
                .children([
                    mono_status("RAM: 14.8 MB".into()),
                    mono_status("FPS: 120".into()),
                    mono_status(format!("Sync Seq: #{}", m.sync.current_seq())),
                ]),
            // Right: network pill + sync + theme toggle
            row()
                .gap(SP3)
                .items_center()
                .shrink0()
                .children([
                    network_pill(net),
                    icon_button(icons::lucide::refresh_cw(), Msg::ManualSync, "Trigger sync"),
                    icon_button(
                        if m.dark_mode {
                            icons::lucide::sun()
                        } else {
                            icons::lucide::moon()
                        },
                        Msg::ToggleTheme,
                        "Toggle theme",
                    ),
                ]),
        ])
}

fn traffic_light(fill: u32, border: u32) -> Element<Msg> {
    div()
        .w(12.0)
        .h(12.0)
        .rounded_full()
        .themed(move |t, s| {
            let _ = t;
            s.bg(Color::from_rgb8(
                (fill >> 16) as u8,
                (fill >> 8) as u8,
                fill as u8,
            ))
            .border(
                1.0,
                Color::from_rgb8((border >> 16) as u8, (border >> 8) as u8, border as u8),
            )
        })
}

fn pill_text(label: &str) -> Element<Msg> {
    div()
        .px(SP2)
        .py(2.0)
        .rounded(R_SM)
        .themed(|t, s| s.bg(t.accent_bg))
        .child(
            text(label)
                .size(TextSize::Xs)
                .mono()
                .weight(Weight::Semibold)
                .themed(|t, s| s.color(t.accent_text)),
        )
}

fn mono_status(s: String) -> Element<Msg> {
    text(s)
        .size(TextSize::Xs)
        .mono()
        .tabular()
        .themed(|t, s| s.color(t.text_muted))
}

fn network_pill(net: NetworkCondition) -> Element<Msg> {
    let label = network_label(net.mode);
    let mode = net.mode;
    div()
        .px(SP2)
        .py(2.0)
        .rounded_full()
        .themed(move |t, s| match mode {
            NetworkMode::Offline => s.bg(t.danger.bg).border(1.0, t.danger.border),
            NetworkMode::PacketLoss | NetworkMode::HighLatency => {
                s.bg(t.warning.bg).border(1.0, t.warning.border)
            }
            _ => s.bg(t.accent_bg).border(1.0, t.accent_border),
        })
        .child(
            text(label)
                .size(TextSize::Xs)
                .mono()
                .themed(move |t, s| match mode {
                    NetworkMode::Offline => s.color(t.danger.text),
                    NetworkMode::PacketLoss | NetworkMode::HighLatency => s.color(t.warning.text),
                    _ => s.color(t.accent_text),
                }),
        )
}

fn icon_button(icon_el: Element<Msg>, msg: Msg, label: &str) -> Element<Msg> {
    div()
        .w(24.0)
        .h(24.0)
        .items_center()
        .justify_center()
        .rounded(R_SM)
        .focusable(true)
        .semantics(Semantics::Button)
        .label(label)
        .on_click(msg)
        .state_layer(|t| t.text)
        .child(icon_el.w(14.0).h(14.0))
}

fn network_label(mode: NetworkMode) -> &'static str {
    match mode {
        NetworkMode::Optimal => "optimal",
        NetworkMode::Wifi => "wifi",
        NetworkMode::Cellular4g => "cellular 4g",
        NetworkMode::HighLatency => "high latency",
        NetworkMode::PacketLoss => "packet loss",
        NetworkMode::Offline => "offline",
    }
}

// ---------- Main row: sidebar + panel ----------

fn main_row(m: &Model) -> Element<Msg> {
    row()
        .grow()
        .overflow_hidden()
        .children([sidebar(m), main_panel(m)])
}

// ---------- Sidebar ----------

fn sidebar(m: &Model) -> Element<Msg> {
    let items = [
        (Tab::Chats, Some(m.total_unread())),
        (Tab::Contacts, None),
        (Tab::Storage, None),
        (Tab::SyncDiag, nonzero(m.outbox_count())),
        (Tab::RustArch, None),
        (Tab::Benchmarks, None),
    ];

    let nav: Vec<Element<Msg>> = items
        .iter()
        .map(|(tab, badge)| nav_item(m, *tab, *badge))
        .collect();

    col()
        .w(64.0)
        .h_full()
        .items_center()
        .pt(SP4)
        .pb(SP4)
        .gap(SP4)
        .shrink0()
        .themed(|t, s| s.bg(t.surface).border_right(1.0, t.border_subtle))
        .children([
            // User avatar block
            avatar_initials(&m.current_user.name, 40.0)
                .semantics(Semantics::Button)
                .label("Open profile")
                .on_click(Msg::OpenProfile),
            // Nav icons
            col().gap(SP4).children(nav),
            spacer(),
            // Engine status chip: breathes — the surface is alive
            div()
                .w(36.0)
                .h(36.0)
                .rounded(R_MD)
                .items_center()
                .justify_center()
                .keyframes(
                    Keyframes::new(2600.0)
                        .themed_stop(0.0, |t, s| s.bg(t.accent_bg).border(1.0, t.accent_border))
                        .themed_stop(
                            0.5,
                            |t, s| s.bg(t.accent_border).border(1.0, t.accent),
                        )
                        .themed_stop(1.0, |t, s| s.bg(t.accent_bg).border(1.0, t.accent_border)),
                )
                .child(icon_named("zap", 16.0)),
        ])
}

fn nav_item(m: &Model, tab: Tab, badge: Option<u32>) -> Element<Msg> {
    let active = m.active_tab == tab;

    let base = div()
        .w(44.0)
        .h(44.0)
        .rounded(R_LG)
        .items_center()
        .justify_center()
        .focusable(true)
        .semantics(Semantics::Button)
        .label(tab.label())
        .on_click(Msg::SelectTab(tab));

    let mut el = if active {
        base.themed(|t, s| s.bg(t.accent)).child(
            icon_named(tab.icon(), 20.0).themed(|t, s| s.color(t.on_accent)),
        )
    } else {
        base.state_layer(|t| t.text)
            .child(icon_named(tab.icon(), 20.0))
    };

    if let Some(count) = badge {
        el = el.child(badge_counter(count));
    }
    el
}

fn badge_counter(count: u32) -> Element<Msg> {
    div()
        .absolute()
        .top(-2.0)
        .right(-2.0)
        .min_w(18.0)
        .h(18.0)
        .px(4.0)
        .rounded_full()
        .items_center()
        .justify_center()
        .themed(|t, s| s.bg(t.danger.solid))
        .child(
            text(if count > 99 { "99+".into() } else { count.to_string() })
                .size(TextSize::Xs)
                .weight(Weight::Semibold)
                .themed(|t, s| s.color(t.on_accent)),
        )
}

fn nonzero(n: usize) -> Option<u32> {
    if n > 0 { Some(n as u32) } else { None }
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn avatar_initials(name: &str, size: f32) -> Element<Msg> {
    div()
        .w(size)
        .h(size)
        .rounded(R_MD)
        .items_center()
        .justify_center()
        .themed(|t, s| s.bg(t.accent_bg).ring(2.0, t.accent_border))
        .child(
            text(initials(name))
                .size(TextSize::Sm)
                .weight(Weight::Semibold)
                .themed(|t, s| s.color(t.accent_text)),
        )
}

/// Icon by kebab-case lucide name; falls back to a dot for names outside the
/// vendored subset (missing icons are a P5 asset task, not a layout risk).
fn icon_named(name: &str, size: f32) -> Element<Msg> {
    let el = crate::icons::by_name(name)
        .or_else(|| icons::lucide::by_name(name))
        .unwrap_or_else(|| {
            // Unknown name: neutral dot placeholder (should not ship).
            div().w(24.0).h(24.0).rounded_full().themed(|t, s| {
                let _ = t;
                s.bg(Color::from_rgb8(120, 120, 130))
            })
        });
    el.w(size).h(size)
}

// ---------- Main panel switcher ----------

fn main_panel(m: &Model) -> Element<Msg> {
    match m.active_tab {
        Tab::Chats => row()
            .grow()
            .overflow_hidden()
            .children([chat_list(m), chat_area(m)]),
        Tab::Storage => crate::views::storage_view(m),
        Tab::SyncDiag => crate::views::sync_diag_view(m),
        Tab::Benchmarks => crate::views::benchmark_view(m),
        Tab::RustArch => crate::views::arch_view(m),
        Tab::Contacts => contacts_view(m),
        Tab::FxGallery => crate::fx::fx_showcase(m),
    }
}

fn contacts_view(m: &Model) -> Element<Msg> {
    col()
        .grow()
        .scroll_y()
        .p(SP6)
        .gap(SP3)
        .themed(|t, s| s.bg(t.bg))
        .children(
            m.contacts.iter().map(|c| contact_row(m, c)).collect::<Vec<_>>(),
        )
}

fn contact_row(m: &Model, c: &Contact) -> Element<Msg> {
    let name = c.name.clone();
    let bio = c.bio.clone();
    let online = c.online_status;
    let contact_id = c.id.clone();
    let _ = m;
    row()
        .w_full()
        .px(SP4)
        .py(SP3)
        .rounded(R_LG)
        .items_center()
        .gap(SP3)
        .semantics(Semantics::Button)
        .label(format!("Start chat with {}", name))
        .on_click(Msg::StartChatWithContact(contact_id))
        .state_layer(|t| t.text)
        .themed(|t, s| s.bg(t.surface))
        .children([
            chat_avatar(&name, 0),
            col().grow().gap(2.0).children([
                text(name).size(TextSize::Sm).weight(Weight::Semibold),
                text(bio).size(TextSize::Xs).truncate().themed(|t, s| s.color(t.text_muted)),
            ]),
            text(match online {
                OnlineStatus::Online => "online",
                OnlineStatus::Busy => "busy",
                OnlineStatus::Offline => "offline",
            })
            .size(TextSize::Xs)
            .mono()
            .themed(move |t, s| match online {
                OnlineStatus::Online => s.color(t.success.text),
                OnlineStatus::Busy => s.color(t.warning.text),
                OnlineStatus::Offline => s.color(t.text_disabled),
            }),
        ])
        .child({
            let _ = m;
            div().w(0.0).h(0.0)
        })
}

// ---------- Chat list (ChatList.tsx) ----------

fn chat_list(m: &Model) -> Element<Msg> {
    let visible = m.visible_conversations();

    col()
        .w(288.0)
        .h_full()
        .shrink0()
        .themed(|t, s| s.bg(t.surface).border_right(1.0, t.border_subtle))
        .children([
            // Search + filter bar
            col()
                .p(SP3)
                .gap(SP2)
                .themed(|t, s| s.border_bottom(1.0, t.border_subtle))
                .children(vec![
                    {
                        let ti: Element<Msg> = text_input(m.search.clone())
                            .placeholder("Search chats & messages...")
                            .width(240.0)
                            .size(ControlSize::Sm)
                            .id("search-chats")
                            .on_input(Msg::SearchInput)
                            .into();
                        ti
                    },
                    row()
                        .gap(SP1)
                        .children(vec![
                            filter_chip("all", m.filter == Filter::All, Msg::FilterAll),
                            filter_chip("unread", m.filter == Filter::Unread, Msg::FilterUnread),
                            filter_chip("pinned", m.filter == Filter::Pinned, Msg::FilterPinned),
                        ]),
                ]),
            // Conversation rows
            col()
                .grow()
                .scroll_y()
                .id("chat-list-scroll")
                .children(visible.iter().map(|c| chat_row(m, c)).collect::<Vec<_>>()),
            // Status footer
            row()
                .p(SP2)
                .justify_between()
                .shrink0()
                .themed(|t, s| s.border_top(1.0, t.border_subtle))
                .children([
                    text(format!("{} Conversations", visible.len()))
                        .size(TextSize::Xs)
                        .mono()
                        .themed(|t, s| s.color(t.text_subtle)),
                    text("Local Store Ready")
                        .size(TextSize::Xs)
                        .mono()
                        .themed(|t, s| s.color(t.success.text)),
                ]),
        ])
}

fn filter_chip(label: &str, active: bool, msg: Msg) -> Element<Msg> {
    let base = div()
        .px(SP2)
        .py(2.0)
        .rounded(R_SM)
        .semantics(Semantics::Button)
        .label(format!("Filter: {label}"))
        .on_click(msg);
    if active {
        base.themed(|t, s| s.bg(t.accent)).child(
            text(label)
                .size(TextSize::Xs)
                .weight(Weight::Medium)
                .themed(|t, s| s.color(t.on_accent)),
        )
    } else {
        base.state_layer(|t| t.text).child(
            text(label)
                .size(TextSize::Xs)
                .weight(Weight::Medium)
                .themed(|t, s| s.color(t.text_muted)),
        )
    }
}

fn chat_row(m: &Model, conv: &Conversation) -> Element<Msg> {
    let active = m.active_conv_id.as_deref() == Some(&conv.id);
    let id = conv.id.clone();
    let pinned = conv.is_pinned;

    let mut row_el = div()
        .w_full()
        .px(SP3)
        .py(SP2)
        .focusable(true)
        .semantics(Semantics::Button)
        .label(format!("Conversation with {}", conv.name))
        .on_click(Msg::SelectConversation(id.clone()))
        .on_right_click(Msg::OpenContextMenu(id))
        .themed(move |t, s| {
            if active {
                s.bg(t.element_active)
            } else if pinned {
                s.bg(t.element)
            } else {
                s
            }
        })
        .state_layer(|t| t.text);

    if m.context_menu_conv.as_deref() == Some(&conv.id) {
        row_el = row_el.child(
            context_menu(vec![
                (
                    if conv.is_pinned { "Unpin Chat" } else { "Pin Chat" }.to_string(),
                    Msg::TogglePin(conv.id.clone()),
                ),
                (
                    if conv.is_muted { "Unmute" } else { "Mute" }.to_string(),
                    Msg::ToggleMute(conv.id.clone()),
                ),
                (
                    "Clear Chat Messages".to_string(),
                    Msg::ClearConversation(conv.id.clone()),
                ),
            ])
            .on_close(Msg::CloseContextMenu),
        );
    }

    row_el
        .child(
            row()
                .gap(SP3)
                .items_start()
                .children([
                    chat_avatar(&conv.name, conv.unread_count),
                    col()
                        .grow()
                        .min_w(0.0)
                        .gap(2.0)
                        .children([
                            row()
                                .justify_between()
                                .items_center()
                                .children([
                                    text(&conv.name)
                                        .size(TextSize::Sm)
                                        .weight(Weight::Semibold)
                                        .truncate()
                                        .grow(),
                                    text(relative_time(
                                        conv.last_message_time,
                                        m.sync.now(),
                                    ))
                                    .size(TextSize::Xs)
                                    .mono()
                                    .themed(|t, s| s.color(t.text_subtle)),
                                ]),
                            row()
                                .justify_between()
                                .items_center()
                                .gap(SP2)
                                .children([
                                    text(
                                        conv.last_message.as_deref().unwrap_or("No messages yet"),
                                    )
                                    .size(TextSize::Xs)
                                    .truncate()
                                    .grow()
                                    .themed(|t, s| s.color(t.text_muted)),
                                    if conv.is_muted {
                                        icon_named("bell-off", 12.0)
                                    } else {
                                        spacer_w(0.0)
                                    },
                                ]),
                        ]),
                ]),
        )
}

fn chat_avatar(name: &str, unread: u32) -> Element<Msg> {
    stack()
        .w(40.0)
        .h(40.0)
        .child(
            div()
                .w(40.0)
                .h(40.0)
                .rounded(R_MD)
                .items_center()
                .justify_center()
                .themed(|t, s| s.bg(t.accent_bg))
                .child(
                    text(initials(name))
                        .size(TextSize::Sm)
                        .weight(Weight::Semibold)
                        .themed(|t, s| s.color(t.accent_text)),
                ),
        )
        .child(if unread > 0 {
            div()
                .absolute()
                .top(-6.0)
                .right(-6.0)
                .min_w(17.0)
                .h(17.0)
                .px(4.0)
                .rounded_full()
                .items_center()
                .justify_center()
                .themed(|t, s| s.bg(t.danger.solid))
                .child(
                    text(if unread > 99 { "99+".into() } else { unread.to_string() })
                        .size(TextSize::Xs)
                        .weight(Weight::Semibold)
                        .themed(|t, s| s.color(t.on_accent)),
                )
        } else {
            div().w(0.0).h(0.0)
        })
}

pub fn relative_time(ts: i64, now: i64) -> String {
    let diff = (now - ts).max(0);
    if diff < 60_000 {
        "now".into()
    } else if diff < 3_600_000 {
        format!("{}m", diff / 60_000)
    } else if diff < 86_400_000 {
        format!("{}h", diff / 3_600_000)
    } else {
        format!("{}d", diff / 86_400_000)
    }
}

// ---------- Chat area ----------

fn chat_area(m: &Model) -> Element<Msg> {
    let Some(conv_id) = &m.active_conv_id else {
        return empty_state();
    };
    let msgs = m.sync.storage().peek_messages_by_conversation(conv_id);
    let conv = m.conversations.iter().find(|c| &c.id == conv_id);

    // Aurora wash sits behind the whole pane (light has a source); the
    // composer is the screen's single glass focal surface floating over it.
    stack()
        .grow()
        .overflow_hidden()
        // BISECT: wash removed
        .child(
            col()
                .grow()
                .overflow_hidden()
                .children([
                    // Header: translucent over the wash, live pulse on sync
                    row()
                        .h(56.0)
                        .px(SP4)
                        .items_center()
                        .gap(SP3)
                        .shrink0()
                        .themed(|t, s| {
                            s.bg(t.surface.with_alpha(0.82))
                                .border_bottom(1.0, t.border_subtle)
                        })
                        .children([
                            chat_avatar(
                                msgs.first()
                                    .map(|msg| msg.sender_name.as_str())
                                    .or(conv.map(|c| c.name.as_str()))
                                    .unwrap_or("Chat"),
                                0,
                            ),
                            col().gap(2.0).children([
                                text(
                                    conv.map(|c| c.name.clone())
                                        .unwrap_or_else(|| "Chat".into()),
                                )
                                .size(TextSize::Sm)
                                .weight(Weight::Semibold),
                                text(format!(
                                    "Rust Sync Sequence: #{}",
                                    conv.map(|c| c.last_sync_seq).unwrap_or(0)
                                ))
                                .size(TextSize::Xs)
                                .mono()
                                .themed(|t, s| s.color(t.text_subtle)),
                            ]),
                            spacer_w(8.0),
                            status("Live", Status::Success).live(true).into(),
                        ]),
                    // Messages feed — stick_to_bottom keeps new sends pinned
                    col()
                        .grow()
                        .scroll_y()
                        .stick_to_bottom()
                        .id("message-feed")
                        .p(SP4)
                        .gap(SP4)
                        .children(
                            msgs.iter()
                                .map(|msg| message_bubble(m, msg))
                                .collect::<Vec<_>>(),
                        ),
                    composer(m),
                ]),
        )
}

fn empty_state() -> Element<Msg> {
    // Glow behind the mark: light with a source (radial accent halo).
    stack()
        .grow()
        .items_center()
        .justify_center()
        .themed(|t, s| s.bg(t.bg))
        .child(crate::fx::radial_glow().absolute())
        .child(
            col()
                .items_center()
                .gap(SP3)
                .children([
                    text("WeChat Desktop (Rust Native)")
                        .size(TextSize::Lg)
                        .weight(Weight::Semibold),
                    text(
                        "Select a conversation to inspect sync sequences & storage compaction",
                    )
                    .size(TextSize::Xs)
                    .themed(|t, s| s.color(t.text_muted)),
                    crate::fx::breathing_chip("ENGINE LIVE".into()),
                ]),
        )
}

fn message_bubble(m: &Model, msg: &Message) -> Element<Msg> {
    let is_new = msg.seq_id > m.initial_max_seq;
    let is_self = msg.is_self;

    // System message: centered pill
    if msg.message_type == MessageType::System {
        return row().justify_center().child(
            div()
                .max_w(420.0)
                .px(SP3)
                .py(SP1)
                .rounded(R_MD)
                .themed(|t, s| s.bg(t.accent_bg).border(1.0, t.accent_border))
                .child(
                    text(&msg.content)
                        .size(TextSize::Xs)
                        .mono()
                        .themed(|t, s| s.color(t.accent_text)),
                ),
        );
    }

    let bubble_body: Element<Msg> = if msg.message_type == MessageType::Code {
        code_card(msg)
    } else {
        text_bubble(msg)
    };

    // Meta line: seq + compression + status
    let meta = row()
        .gap(SP1)
        .px(4.0)
        .children(meta_items(msg));

    let mut content_col = col().gap(2.0).max_w(600.0).child(bubble_body);
    if !is_self {
        content_col = content_col.child(
            text(&msg.sender_name)
                .size(TextSize::Xs)
                .themed(|t, s| s.color(t.text_subtle)),
        );
    }
    content_col = content_col.child(meta);

    let avatar = chat_avatar(&msg.sender_name, 0);

    let row = if is_self {
        row()
            .justify_end()
            .gap(SP2)
            .items_start()
            .children([content_col, avatar])
    } else {
        row().gap(SP2).items_start().children([avatar, content_col])
    };
    // Choreography only for messages that arrive after load: motion conveys
    // state change, never page load (product register).
    if is_new {
        crate::fx::animated_bubble(row, &format!("msg-{}", msg.id))
    } else {
        row
    }
}

fn text_bubble(msg: &Message) -> Element<Msg> {
    let is_self = msg.is_self;
    div()
        .max_w(520.0)
        .px(SP3)
        .py(SP2)
        .rounded(R_XL)
        .themed(move |t, s| {
            if is_self {
                s.bg(t.accent)
            } else {
                s.bg(t.element_hover)
            }
        })
        .child(text(&msg.content).size(TextSize::Sm).themed(move |t, s| {
            if is_self {
                s.color(t.on_accent)
            } else {
                s.color(t.text)
            }
        }))
}

fn code_card(msg: &Message) -> Element<Msg> {
    let snippet = msg.code_snippet.clone().unwrap_or_else(|| msg.content.clone());
    // col(), not div(): div is a ROW in fenestra — multi-child vertical
    // stacking needs an explicit column. Width lives here so the inner
    // header row can justify_between across it.
    col()
        .w_full()
        .max_w(560.0)
        .rounded(R_LG)
        .overflow_hidden()
        .themed(|t, s| s.bg(t.bg).border(1.0, t.border))
        .children([
            row()
                .w_full()
                .px(SP3)
                .py(SP1)
                .justify_between()
                .items_center()
                .themed(|t, s| s.border_bottom(1.0, t.border_subtle))
                .children([
                    text(msg.code_language.clone().unwrap_or_else(|| "rust".into()))
                        .size(TextSize::Xs)
                        .mono()
                        .weight(Weight::Semibold)
                        .themed(|t, s| s.color(t.accent_text)),
                    text("Copy").size(TextSize::Xs).mono(),
                ]),
            text(snippet)
                .size(TextSize::Xs)
                .mono()
                .p(SP3)
                .themed(|t, s| s.color(t.success.text)),
        ])
}

fn meta_items(msg: &Message) -> Vec<Element<Msg>> {
    let mut items = vec![
        text(format!("Seq #{}", msg.seq_id))
            .size(TextSize::Xs)
            .mono()
            .themed(|t, s| s.color(t.text_disabled)),
    ];
    if let Some(bytes) = msg.compressed_bytes {
        items.push(
            text(format!("{bytes}B LZ4"))
                .size(TextSize::Xs)
                .mono()
                .themed(|t, s| s.color(t.text_disabled)),
        );
    }
    if msg.is_self {
        let status = msg.status;
        let status_label = match status {
            SyncStatus::Sending => "[Sending]",
            SyncStatus::Pending => "[Outbox]",
            SyncStatus::Failed => "[Failed]",
            SyncStatus::Synced => "[Synced]",
        };
        items.push(
            text(status_label.to_string())
                .size(TextSize::Xs)
                .mono()
                .themed(move |t, s| match status {
                    SyncStatus::Pending | SyncStatus::Sending => s.color(t.warning.text),
                    SyncStatus::Failed => s.color(t.danger.text),
                    SyncStatus::Synced => s.color(t.success.text),
                }),
        );
    }
    items
}

// ---------- Composer ----------

fn composer(m: &Model) -> Element<Msg> {
    // The screen's one glass surface: vibrancy tint over the aurora wash,
    // specular rim from the Glass material. Legibility protected by fill alpha.
    col()
        .shrink0()
        .surface(Surface::Glass)
        .children([
            // Toolbar hint row
            row()
                .px(SP3)
                .py(SP1)
                .justify_between()
                .items_center()
                .children([
                    row()
                        .gap(SP2)
                        .children([
                            {
                                // Emoji anchor: popover toggles below the smiley.
                                let mut anchor = div();
                                anchor = anchor.child(
                                    icon_button(icons::lucide::star(), Msg::ToggleEmojiPicker, "Emoji picker"),
                                );
                                if m.emoji_picker_open {
                                    anchor = anchor.child(emoji_popover());
                                }
                                anchor
                            },
                            icon_button(icons::lucide::file(), Msg::ComposerSend, "Attach file (P4)"),
                            icon_button(icons::lucide::copy(), Msg::ComposerSend, "Code snippet (P4)"),
                        ])
                        .shrink0(),
                    text("Enter to send, Shift+Enter for new line")
                        .size(TextSize::Xs)
                        .mono()
                        .themed(|t, s| s.color(t.text_subtle)),
                ]),
            // Input row
            {
                let ta: Element<Msg> = text_area(m.composer.clone())
                    .placeholder("Type a message or paste code...")
                    .width(480.0)
                    .min_height(40.0)
                    .id("composer")
                    .on_input(Msg::ComposerInput)
                    .into();
                row()
                    .px(SP3)
                    .pb(SP2)
                    .gap(SP2)
                    .items_end()
                    .children([ta.grow(), send_button(!m.composer.trim().is_empty())])
            }
        ])
}

fn send_button(enabled: bool) -> Element<Msg> {
    if enabled {
        button("Send").size(ControlSize::Sm).on_click(Msg::ComposerSend).into()
    } else {
        button("Send")
            .size(ControlSize::Sm)
            .disabled(true)
            .on_click(Msg::ComposerSend)
            .into()
    }
}

fn spacer_w(w: f32) -> Element<Msg> {
    div().w(w).h(0.0)
}

fn profile_modal(m: &Model) -> Element<Msg> {
    let ti_name: Element<Msg> = text_input(m.profile_name.clone())
        .placeholder("Display name")
        .width(360.0)
        .id("profile-name")
        .on_input(Msg::ProfileNameInput)
        .into();
    let ta_bio: Element<Msg> = text_area(m.profile_bio.clone())
        .placeholder("Bio")
        .width(360.0)
        .min_height(60.0)
        .id("profile-bio")
        .on_input(Msg::ProfileBioInput)
        .into();

    modal("Profile & Settings")
        .max_width(420.0)
        .id("profile-modal")
        .on_close(Msg::CloseProfile)
        .child(
            row().gap(SP3).items_center().children([
                avatar_initials(&m.current_user.name, 56.0),
                col().gap(2.0).children([
                    text(format!("WeChat ID: {}", m.current_user.wechat_id))
                        .size(TextSize::Xs)
                        .mono()
                        .themed(|t, s| s.color(t.text_muted)),
                    text(format!("Region: {}", m.current_user.region))
                        .size(TextSize::Xs)
                        .mono()
                        .themed(|t, s| s.color(t.text_muted)),
                ]),
            ]),
        )
        .child(col().gap(SP1).child(ti_name))
        .child(ta_bio)
        .child(
            row()
                .justify_end()
                .gap(SP2)
                .child(
                    button("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm)
                        .on_click(Msg::CloseProfile),
                )
                .child(
                    button("Save")
                        .size(ControlSize::Sm)
                        .on_click(Msg::SaveProfile),
                ),
        )
        .into()
}

const COMMON_EMOJIS: [&str; 15] = [
    "\u{1F44D}", "\u{1F389}", "\u{1F980}", "\u{1F525}", "\u{1F680}",
    "\u{2764}", "\u{1F604}", "\u{1F64F}", "\u{26A1}", "\u{2615}",
    "\u{1F4A1}", "\u{1F4AF}", "\u{1F44F}", "\u{1F916}", "\u{1F4E6}",
];

fn emoji_popover() -> Element<Msg> {
    col()
        .p(SP2)
        .gap(SP1)
        .w(200.0)
        .surface(Surface::Popover)
        .overlay(Overlay::menu())
        .on_close(Msg::ToggleEmojiPicker)
        .child(
            row()
                .wrap()
                .gap(SP1)
                .children(
                    COMMON_EMOJIS
                        .iter()
                        .map(|e| emoji_cell(e.to_string()))
                        .collect::<Vec<_>>(),
                ),
        )
}

fn emoji_cell(emoji: String) -> Element<Msg> {
    let label = emoji.clone();
    div()
        .w(32.0)
        .h(32.0)
        .rounded(R_MD)
        .items_center()
        .justify_center()
        .semantics(Semantics::Button)
        .label(format!("Emoji {emoji}"))
        .on_click(Msg::EmojiPicked(emoji))
        .state_layer(|t| t.text)
        .child(text(label).size(TextSize::Lg))
}
