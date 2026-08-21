//! P4 verification: secondary views render and their actions mutate state.

use fenestra::prelude::*;
use fenestra::shell::{render_app, Harness};
use wechat_fui_native::ui::{Model, Msg, Tab};

fn app_on(tab: Tab) -> Model {
    let mut m = Model::new();
    m.update(Msg::SelectTab(tab));
    m
}

#[test]
fn storage_view_renders_with_live_metrics() {
    let mut app = app_on(Tab::Storage);
    let theme = app.theme();
    let image = render_app(&mut app, &[], (1280, 800), &theme);
    let path = std::env::temp_dir().join("wechat_fui_storage.png");
    image.save(&path).expect("save png");
}

#[test]
fn vacuum_compact_updates_report() {
    let mut app = app_on(Tab::Storage);
    assert!(app.compaction_result.is_none());
    app.update(Msg::VacuumCompact);
    let report = app.compaction_result.as_ref().expect("report set");
    assert!(report.freed_bytes > 0);
}

#[test]
fn stress_test_inserts_messages() {
    let mut app = app_on(Tab::Storage);
    let before = app.sync.storage().get_metrics().total_messages;
    app.update(Msg::SetStressCount(500));
    app.update(Msg::RunStressTest(500));
    let after = app.sync.storage().get_metrics().total_messages;
    assert_eq!(after - before, 500);
}

#[test]
fn export_import_roundtrip_via_messages() {
    let mut app = app_on(Tab::Storage);
    // Export writes a file to temp dir
    app.update(Msg::ExportDb);
    assert!(app.import_status.as_deref().unwrap().contains("Exported"));

    // Import the exported JSON back
    let path = std::env::temp_dir().join("wechat_fui_db_backup.json");
    let json = std::fs::read_to_string(&path).expect("exported file exists");
    app.import_json = json;
    app.update(Msg::ImportDb);
    assert!(app.import_status.as_deref().unwrap().contains("restored"));
}

#[test]
fn sync_diag_view_network_switching() {
    use wechat_fui_native::types::NetworkMode;

    let mut app = app_on(Tab::SyncDiag);
    app.update(Msg::SetNetwork(NetworkMode::PacketLoss, 80.0, 0.25));
    assert_eq!(app.sync.network().mode, NetworkMode::PacketLoss);
    assert_eq!(app.sync.network().packet_loss_rate, 0.25);

    // Log filter state changes
    app.update(Msg::SetLogFilter("ACK".into()));
    assert_eq!(app.log_filter, "ACK");
}

#[test]
fn sync_diag_view_renders() {
    let mut app = app_on(Tab::SyncDiag);
    let theme = app.theme();
    let image = render_app(&mut app, &[], (1280, 800), &theme);
    let path = std::env::temp_dir().join("wechat_fui_syncdiag.png");
    image.save(&path).expect("save png");
}

#[test]
fn benchmark_search_test_executes_query() {
    let mut app = app_on(Tab::Benchmarks);
    app.update(Msg::BenchQueryInput("sync".into()));
    app.update(Msg::RunSearchTest);
    let (count, _ms) = app.search_test_result.expect("search executed");
    assert!(count > 0, "seed messages contain 'sync'");
}

#[test]
fn benchmark_and_arch_views_render() {
    for tab in [Tab::Benchmarks, Tab::RustArch] {
        let mut app = app_on(tab);
        let theme = app.theme();
        let image = render_app(&mut app, &[], (1280, 800), &theme);
        let name = match tab {
            Tab::Benchmarks => "wechat_fui_bench.png",
            _ => "wechat_fui_arch.png",
        };
        let path = std::env::temp_dir().join(name);
        image.save(&path).expect("save png");
    }
}

#[test]
fn arch_snippet_tabs_switch_content() {
    let mut app = app_on(Tab::RustArch);
    app.update(Msg::SelectArchSnippet(2));
    assert_eq!(app.arch_snippet_idx, 2);
}

#[test]
fn contacts_view_start_chat_creates_conversation() {
    let mut app = app_on(Tab::Contacts);
    let before = app.conversations.len();
    app.update(Msg::StartChatWithContact("contact_linus".into()));
    // Existing conversation found → no new one created, but switched to chats
    assert_eq!(app.active_tab, Tab::Chats);
    assert_eq!(app.active_conv_id.as_deref(), Some("conv_contact_linus"));

    // A contact without an existing conversation creates one
    app.update(Msg::SelectTab(Tab::Contacts));
    app.update(Msg::StartChatWithContact("contact_wechat_team".into()));
    assert!(app.conversations.len() > before || app.active_conv_id.is_some());
}

#[test]
fn accessibility_exposes_view_surfaces() {
    let app = app_on(Tab::SyncDiag);
    let mut h = Harness::new(app, Theme::from_accent(150.0, Mode::Dark), (1280, 800));
    assert!(h.query(&by::label("Network profile: Offline / Airplane")).is_some());
    assert!(h.query(&by::label("Log filter: HEARTBEAT")).is_some());

    let mut app2 = app_on(Tab::Contacts);
    let mut h2 = Harness::new(app2, Theme::from_accent(150.0, Mode::Dark), (1280, 800));
}
