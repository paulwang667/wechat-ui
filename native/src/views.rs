//! Secondary views ported from StorageOptimizerView / SyncDiagnosticsView /
//! PerformanceBenchmarkView / RustArchitectureView.

use crate::types::*;
use crate::ui::{Msg, Model};
use fenestra::prelude::*;

// ---------- Shared building blocks ----------

fn view_shell(title: &str, pill: &str, subtitle: &str, children: Vec<Element<Msg>>) -> Element<Msg> {
    col()
        .grow()
        .scroll_y()
        .id("view-scroll")
        .p(SP6)
        .gap(SP6)
        .themed(|t, s| s.bg(t.bg))
        .children([
            // Header
            row()
                .justify_between()
                .items_center()
                .pb(SP4)
                .shrink0()
                .themed(|t, s| s.border_bottom(1.0, t.border_subtle))
                .children([
                    col().gap(2.0).children([
                        row()
                            .gap(SP2)
                            .items_center()
                            .children([
                                text(title).size(TextSize::Lg).weight(Weight::Semibold),
                                pill_text_small(pill),
                            ]),
                        text(subtitle)
                            .size(TextSize::Xs)
                            .themed(|t, s| s.color(t.text_subtle)),
                    ]),
                ]),
        ])
        .children(children)
}

fn pill_text_small(label: &str) -> Element<Msg> {
    div()
        .px(SP2)
        .py(2.0)
        .rounded_full()
        .themed(|t, s| s.bg(t.accent_bg))
        .child(
            text(label)
                .size(TextSize::Xs)
                .mono()
                .weight(Weight::Semibold)
                .themed(|t, s| s.color(t.accent_text)),
        )
}

/// Metric card: label row, big mono value, optional suffix chip + footnote.
fn metric_card(
    label: &str,
    value: String,
    value_color: ValueColor,
    chip: Option<String>,
    footnote: String,
) -> Element<Msg> {
    let mut value_row = row().gap(SP2).items_baseline().children([
        text(value)
            .size(TextSize::Xl)
            .weight(Weight::Semibold)
            .mono()
            .tabular()
            .themed(move |t, s| match value_color {
                ValueColor::Text => s.color(t.text),
                ValueColor::Accent => s.color(t.accent_text),
                ValueColor::Warning => s.color(t.warning.text),
                ValueColor::Danger => s.color(t.danger.text),
            }),
    ]);
    if let Some(chip_text) = chip {
        value_row = value_row.child(
            div()
                .px(SP1)
                .py(2.0)
                .rounded(R_SM)
                .themed(|t, s| s.bg(t.accent_bg))
                .child(
                    text(chip_text)
                        .size(TextSize::Xs)
                        .mono()
                        .weight(Weight::Semibold)
                        .themed(|t, s| s.color(t.accent_text)),
                ),
        );
    }

    card_base()
        .gap(SP1)
        .children([
            text(label.to_string())
                .size(TextSize::Sm)
                .themed(|t, s| s.color(t.text_muted)),
            value_row,
            text(footnote)
                .size(TextSize::Xs)
                .themed(|t, s| s.color(t.text_subtle)),
        ])
}

fn card_base() -> Element<Msg> {
    div()
        .p(SP4)
        .rounded(R_LG)
        .themed(|t, s| s.bg(t.surface).border(1.0, t.border_subtle))
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Danger reserved for failed-benchmark states
enum ValueColor {
    Text,
    Accent,
    Warning,
    Danger,
}

fn format_kb(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// ---------- Storage Optimizer (StorageOptimizerView.tsx) ----------

pub fn storage_view(m: &Model) -> Element<Msg> {
    let metrics = m.sync.storage().get_metrics();

    view_shell(
        "Local Lightweight Storage Optimizer",
        "Rust B-Tree + LZ4 + WAL",
        "Zero-copy message streaming, memory-mapped I/O, and blob deduplication.",
        vec![
            // Top metric cards
            row()
                .gap(SP4)
                .children([
                    metric_card(
                        "Total Messages",
                        format!("{}", metrics.total_messages),
                        ValueColor::Text,
                        None,
                        format!("in {} chats • columnar index < 0.05ms", metrics.total_conversations),
                    ),
                    metric_card(
                        "LZ4 Compressed Store",
                        format_kb(metrics.compressed_storage_bytes),
                        ValueColor::Accent,
                        Some(format!("-{:.1}%", metrics.compression_ratio)),
                        format!(
                            "Raw: {} (Saved {})",
                            format_kb(metrics.raw_storage_bytes),
                            format_kb(metrics.saved_storage_bytes)
                        ),
                    ),
                    metric_card(
                        "WAL Journal & Index",
                        format_kb(metrics.wal_log_size_bytes + metrics.index_size_bytes),
                        ValueColor::Text,
                        None,
                        format!(
                            "WAL: {} • Index: {}",
                            format_kb(metrics.wal_log_size_bytes),
                            format_kb(metrics.index_size_bytes)
                        ),
                    ),
                    metric_card(
                        "Hot Memory Cache",
                        format_kb(metrics.active_memory_cache_bytes),
                        ValueColor::Text,
                        Some(format!("{:.1}% Hit", metrics.cache_hit_ratio)),
                        "Zero-copy LRU active session buffer".into(),
                    ),
                ])
                .wrap(),
            // Compaction + stress row
            row().gap(SP6).children([
                compaction_card(m, metrics.last_compaction_time).grow(),
                col()
                    .gap(SP4)
                    .grow()
                    .children([stress_card(m), backup_card(m)]),
            ]),
        ],
    )
}

fn compaction_card(m: &Model, last_compaction: i64) -> Element<Msg> {
    let result = m.compaction_result.clone();
    col()
        .p(SP5)
        .gap(SP4)
        .rounded(R_LG)
        .themed(|t, s| s.bg(t.surface).border(1.0, t.border_subtle))
        .children([
            row()
                .justify_between()
                .items_center()
                .children([
                    text("Auto Vacuum & Compaction")
                        .size(TextSize::Sm)
                        .weight(Weight::Semibold),
                    text(format!(
                        "Last compact: {}",
                        crate::ui::relative_time(last_compaction, m.sync.now())
                    ))
                    .size(TextSize::Xs)
                    .mono()
                    .themed(|t, s| s.color(t.text_subtle)),
                ]),
            text(
                "Trims fragmented pages, truncates WAL log journals, and repacks B-Tree tables into continuous disk blocks.",
            )
            .size(TextSize::Sm)
            .themed(|t, s| s.color(t.text_muted)),
            // Multi-tier storage bar
            storage_bar(&m.sync.storage().get_metrics()),
        ])
        .child(match result {
            Some(r) => div()
                .p(SP3)
                .rounded(R_MD)
                .themed(|t, s| s.bg(t.accent_bg).border(1.0, t.accent_border))
                .children([
                    text("Compaction Completed")
                        .size(TextSize::Xs)
                        .weight(Weight::Semibold)
                        .themed(|t, s| s.color(t.accent_text)),
                    text(format!(
                        "Freed: {} in {:.2}ms",
                        format_kb(r.freed_bytes),
                        r.duration_ms
                    ))
                    .size(TextSize::Xs)
                    .mono()
                    .themed(|t, s| s.color(t.accent_text)),
                ]),
            None => spacer_w0(),
        })
        .child(action_button("Run PRAGMA VACUUM", Msg::VacuumCompact, true))
}

fn storage_bar(metrics: &StorageMetrics) -> Element<Msg> {
    let total = metrics
        .compressed_storage_bytes
        .max(1)
        + metrics.index_size_bytes
        + metrics.wal_log_size_bytes
        + metrics.active_memory_cache_bytes;
    let seg = |bytes: u64, color: SegColor| {
        div()
            .h(14.0)
            .w((bytes as f32 / total as f32 * 100.0).max(8.0))
            .themed(move |t, s| match color {
                SegColor::Accent => s.bg(t.accent),
                SegColor::Info => s.bg(t.success.solid),
                SegColor::Warn => s.bg(t.warning.solid),
                SegColor::Muted => s.bg(t.element_hover),
            })
    };
    col().gap(SP2).children([
        row()
            .w_full()
            .overflow_hidden()
            .rounded_full()
            .children(vec![
                seg(metrics.compressed_storage_bytes, SegColor::Accent),
                seg(metrics.index_size_bytes, SegColor::Info),
                seg(metrics.wal_log_size_bytes, SegColor::Warn),
                seg(metrics.active_memory_cache_bytes, SegColor::Muted),
            ]),
        row()
            .gap(SP4)
            .wrap()
            .children(vec![
                legend_dot("Payloads", format_kb(metrics.compressed_storage_bytes), SegColor::Accent),
                legend_dot("Indices", format_kb(metrics.index_size_bytes), SegColor::Info),
                legend_dot("WAL", format_kb(metrics.wal_log_size_bytes), SegColor::Warn),
                legend_dot("Memory Pool", format_kb(metrics.active_memory_cache_bytes), SegColor::Muted),
            ]),
    ])
}

#[derive(Debug, Clone, Copy)]
enum SegColor {
    Accent,
    Info,
    Warn,
    Muted,
}

fn legend_dot(label: &str, size: String, color: SegColor) -> Element<Msg> {
    row()
        .gap(SP1)
        .items_center()
        .children([
            div().w(10.0).h(10.0).rounded_full().themed(move |t, s| match color {
                SegColor::Accent => s.bg(t.accent),
                SegColor::Info => s.bg(t.success.solid),
                SegColor::Warn => s.bg(t.warning.solid),
                SegColor::Muted => s.bg(t.element_hover),
            }),
            text(format!("{label} ({size})"))
                .size(TextSize::Xs)
                .mono()
                .themed(|t, s| s.color(t.text_muted)),
        ])
}

fn stress_card(m: &Model) -> Element<Msg> {
    let counts = [500usize, 2000, 5000, 10000];
    col()
        .p(SP5)
        .gap(SP4)
        .rounded(R_LG)
        .themed(|t, s| s.bg(t.surface).border(1.0, t.border_subtle))
        .children([
            text("Storage Stress & Benchmark Generator")
                .size(TextSize::Sm)
                .weight(Weight::Semibold),
            text("Generate synthetic messages to test local B-Tree insert throughput and virtualized UI rendering.")
                .size(TextSize::Xs)
                .themed(|t, s| s.color(t.text_muted)),
            row()
                .gap(SP2)
                .items_center()
                .children(vec![
                    text("Count:")
                        .size(TextSize::Xs)
                        .themed(|t, s| s.color(t.text_muted)),
                ])
                .children(counts.iter().map(|&n| {
                    let active = m.stress_count == n;
                    let base = div()
                        .px(SP2)
                        .py(2.0)
                        .rounded(R_MD)
                        .semantics(Semantics::Button)
                        .label(format!("Stress count {n}"))
                        .on_click(Msg::SetStressCount(n));
                    if active {
                        base.themed(|t, s| s.bg(t.accent)).child(
                            text(format!("+{n}"))
                                .size(TextSize::Xs)
                                .mono()
                                .weight(Weight::Semibold)
                                .themed(|t, s| s.color(t.on_accent)),
                        )
                    } else {
                        base.state_layer(|t| t.text).child(
                            text(format!("+{n}"))
                                .size(TextSize::Xs)
                                .mono()
                                .themed(|t, s| s.color(t.text_muted)),
                        )
                    }
                }).collect::<Vec<_>>()),
            action_button(
                &format!("Batch Insert {} Messages", m.stress_count),
                Msg::RunStressTest(m.stress_count),
                !m.is_busy,
            ),
        ])
}

fn backup_card(m: &Model) -> Element<Msg> {
    col()
        .p(SP5)
        .gap(SP4)
        .rounded(R_LG)
        .themed(|t, s| s.bg(t.surface).border(1.0, t.border_subtle))
        .children([
            text("Database Backup & Snapshot")
                .size(TextSize::Sm)
                .weight(Weight::Semibold),
            action_button("Export DB Backup (.json)", Msg::ExportDb, true),
            {
                let ta: Element<Msg> = text_area(m.import_json.clone())
                    .placeholder("Paste JSON database snapshot to restore...")
                    .width(280.0)
                    .min_height(40.0)
                    .id("import-json")
                    .on_input(Msg::ImportJsonInput)
                    .into();
                ta
            },
        ])
        .child(match &m.import_status {
            Some(status) => text(status.clone())
                .size(TextSize::Xs)
                .mono()
                .themed(|t, s| s.color(t.accent_text)),
            None => spacer_w0(),
        })
        .child(action_button(
            "Restore Snapshot",
            Msg::ImportDb,
            !m.import_json.trim().is_empty(),
        ))
}

fn action_button(label: &str, msg: Msg, enabled: bool) -> Element<Msg> {
    if enabled {
        button(label.to_string())
            .size(ControlSize::Sm)
            .on_click(msg)
            .into()
    } else {
        button(label.to_string())
            .size(ControlSize::Sm)
            .disabled(true)
            .on_click(msg)
            .into()
    }
}

fn spacer_w0() -> Element<Msg> {
    div().w(0.0).h(0.0)
}

// ---------- Sync Diagnostics (SyncDiagnosticsView.tsx) ----------

const NETWORK_PROFILES: [(NetworkMode, &str, f64, f64, &str); 6] = [
    (NetworkMode::Optimal, "Optimal LAN", 8.0, 0.0, "Ultra-low latency, 0% packet loss"),
    (NetworkMode::Wifi, "Standard Wi-Fi", 25.0, 0.0, "Typical desktop office connection"),
    (NetworkMode::Cellular4g, "Cellular 4G", 90.0, 0.01, "Moderate latency, occasional jitter"),
    (NetworkMode::HighLatency, "High Latency (450ms)", 450.0, 0.02, "Cross-border / satellite link"),
    (NetworkMode::PacketLoss, "Flaky Packet Loss (25%)", 80.0, 0.25, "Simulates dropped ACKs & auto-retry"),
    (NetworkMode::Offline, "Offline / Airplane", 9999.0, 1.0, "Messages held in Outbox queue"),
];

pub fn sync_diag_view(m: &Model) -> Element<Msg> {
    let net = m.sync.network();
    let outbox = m.sync.outbox();

    view_shell(
        "Incremental Sequence Sync Protocol Engine",
        "Protobuf Delta Sync",
        "Monotonic sequence ordering, vector clock convergence, and async outbox dispatcher.",
        vec![
            // Status cards
            row()
                .gap(SP4)
                .wrap()
                .children(vec![
                    metric_card(
                        "Monotonic Sequence ID",
                        format!("#{}", m.sync.current_seq()),
                        ValueColor::Accent,
                        None,
                        "Strict total ordering invariant".into(),
                    ),
                    metric_card(
                        "Outbox Pending Queue",
                        format!("{}", outbox.len()),
                        ValueColor::Text,
                        None,
                        if outbox.is_empty() {
                            "All messages ACKed".into()
                        } else {
                            "Waiting for network sync".into()
                        },
                    ),
                    metric_card(
                        "Current RTT Latency",
                        if net.mode == NetworkMode::Offline {
                            "∞ ms".into()
                        } else {
                            format!("{} ms", net.latency_ms)
                        },
                        ValueColor::Accent,
                        None,
                        format!("Loss rate: {:.0}%", net.packet_loss_rate * 100.0),
                    ),
                    metric_card(
                        "Sync Channel Mode",
                        network_mode_label(net.mode).to_uppercase(),
                        ValueColor::Text,
                        None,
                        "Async worker active".into(),
                    ),
                ]),
            // Network simulator
            card_base()
                .gap(SP3)
                .children([
                    text("Network Condition Simulator (Test Sync Resilience & Auto-Retry)")
                        .size(TextSize::Sm)
                        .weight(Weight::Semibold),
                ])
                .child(row().gap(SP3).wrap().children(
                    NETWORK_PROFILES
                        .iter()
                        .map(|p| network_profile_chip(m, *p))
                        .collect::<Vec<_>>(),
                )),
            // Outbox section when non-empty
            if !outbox.is_empty() {
                outbox_panel(m, &outbox)
            } else {
                spacer_w0()
            },
            // Logs
            logs_panel(m),
        ],
    )
}

fn network_profile_chip(
    m: &Model,
    profile: (NetworkMode, &'static str, f64, f64, &'static str),
) -> Element<Msg> {
    let (mode, label, latency, loss, desc) = profile;
    let is_current = m.sync.network().mode == mode;

    div()
        .p(SP3)
        .rounded(R_LG)
        .min_w(200.0)
        .semantics(Semantics::Button)
        .label(format!("Network profile: {label}"))
        .on_click(Msg::SetNetwork(mode, latency, loss))
        .themed(move |t, s| {
            if is_current {
                s.bg(t.accent_bg).border(1.0, t.accent_border)
            } else {
                s.bg(t.surface).border(1.0, t.border)
            }
        })
        .state_layer(|t| t.text)
        .children([
            text(label.to_string())
                .size(TextSize::Xs)
                .weight(Weight::Semibold),
            text(desc.to_string())
                .size(TextSize::Xs)
                .themed(|t, s| s.color(t.text_muted)),
            row()
                .justify_between()
                .themed(|t, s| s.border_top(1.0, t.border_subtle))
                .children([
                    text(if latency >= 9999.0 {
                        "RTT: Offline".to_string()
                    } else {
                        format!("RTT: {latency}ms")
                    })
                    .size(TextSize::Xs)
                    .mono()
                    .themed(|t, s| s.color(t.text_subtle)),
                    text(format!("Loss: {:.0}%", loss * 100.0))
                        .size(TextSize::Xs)
                        .mono()
                        .themed(|t, s| s.color(t.text_subtle)),
                ]),
        ])
}

fn outbox_panel(_m: &Model, outbox: &[Message]) -> Element<Msg> {
    card_base()
        .gap(SP3)
        .themed(|t, s| s.bg(t.warning.bg).border(1.0, t.warning.border))
        .child(
            row()
                .justify_between()
                .items_center()
                .children([
                    text(format!(
                        "Outbox Queue ({} pending unacked messages)",
                        outbox.len()
                    ))
                    .size(TextSize::Xs)
                    .weight(Weight::Semibold)
                    .themed(|t, s| s.color(t.warning.text)),
                    button("Force Flush Outbox")
                        .size(ControlSize::Sm)
                        .on_click(Msg::FlushOutbox)
                        .into(),
                ]),
        )
        .child(col().gap(SP1).children(
            outbox
                .iter()
                .map(|msg| {
                    row()
                        .p(SP2)
                        .justify_between()
                        .themed(|t, s| s.bg(t.element))
                        .children([
                            text(format!("Seq #{}: {}", msg.seq_id, msg.content))
                                .size(TextSize::Xs)
                                .mono()
                                .truncate()
                                .grow(),
                            text(format!("[{}]", status_label(msg.status)))
                                .size(TextSize::Xs)
                                .mono()
                                .weight(Weight::Semibold)
                                .themed(|t, s| s.color(t.warning.text)),
                        ])
                })
                .collect::<Vec<_>>(),
        ))
}

fn status_label(s: SyncStatus) -> &'static str {
    match s {
        SyncStatus::Sending => "SENDING",
        SyncStatus::Pending => "PENDING",
        SyncStatus::Failed => "FAILED",
        SyncStatus::Synced => "SYNCED",
    }
}

fn logs_panel(m: &Model) -> Element<Msg> {
    let filters = ["ALL", "PUSH_MSG", "PULL_REQ", "PULL_RESP", "ACK", "HEARTBEAT"];
    let all_logs = m.sync.sync_logs();
    let filtered: Vec<&SyncLogEntry> = all_logs
        .iter()
        .filter(|log| {
            m.log_filter == "ALL" || log.log_type.as_str() == m.log_filter
        })
        .collect();

    card_base()
        .gap(SP4)
        .children([
            row()
                .justify_between()
                .items_center()
                .themed(|t, s| s.border_bottom(1.0, t.border_subtle))
                .children([
                    text("Sync Protocol Event Logs")
                        .size(TextSize::Sm)
                        .weight(Weight::Semibold),
                    row()
                        .gap(SP1)
                        .children(filters.iter().map(|f| log_filter_chip(m, f)).collect::<Vec<_>>()),
                ]),
        ])
        .child(if filtered.is_empty() {
            text("No logs for selected filter.")
                .size(TextSize::Xs)
                .themed(|t, s| s.color(t.text_subtle))
        } else {
            col()
                .gap(SP1)
                .children(filtered.iter().take(50).map(|log| log_row(log)).collect::<Vec<_>>())
        })
        .child(spacer_w0())
}

fn log_filter_chip(m: &Model, filter: &'static str) -> Element<Msg> {
    let active = m.log_filter == filter;
    let base = div()
        .px(SP2)
        .py(2.0)
        .rounded(R_SM)
        .semantics(Semantics::Button)
        .label(format!("Log filter: {filter}"))
        .on_click(Msg::SetLogFilter(filter.to_string()));
    if active {
        base.themed(|t, s| s.bg(t.accent)).child(
            text(filter.to_string())
                .size(TextSize::Xs)
                .mono()
                .weight(Weight::Semibold)
                .themed(|t, s| s.color(t.on_accent)),
        )
    } else {
        base.state_layer(|t| t.text).child(
            text(filter.to_string())
                .size(TextSize::Xs)
                .mono()
                .themed(|t, s| s.color(t.text_muted)),
        )
    }
}

fn log_row(log: &SyncLogEntry) -> Element<Msg> {
    let log_type = log.log_type;
    let details = log.details.clone();
    let ts = log.timestamp;
    let _ = &log.id;
    row()
        .p(SP2)
        .gap(SP2)
        .items_start()
        .themed(|t, s| s.bg(t.element))
        .children([
            text(crate::mock::format_clock(ts))
                .size(TextSize::Xs)
                .mono()
                .shrink0()
                .themed(|t, s| s.color(t.text_subtle)),
            text(log_type.as_str().to_string())
                .size(TextSize::Xs)
                .mono()
                .weight(Weight::Semibold)
                .px(SP1)
                .rounded(R_SM)
                .themed(move |t, s| match log_type {
                    SyncLogType::Ack => s.bg(t.accent_bg).color(t.accent_text),
                    SyncLogType::PushMsg => s.bg(t.success.bg).color(t.success.text),
                    SyncLogType::PullResp => s.bg(t.warning.bg).color(t.warning.text),
                    _ => s.bg(t.element_hover).color(t.text_muted),
                }),
            text(details)
                .size(TextSize::Xs)
                .mono()
                .truncate()
                .grow()
                .themed(|t, s| s.color(t.text_muted)),
        ])
}

fn network_mode_label(mode: NetworkMode) -> &'static str {
    match mode {
        NetworkMode::Optimal => "optimal",
        NetworkMode::Wifi => "wifi",
        NetworkMode::Cellular4g => "cellular 4g",
        NetworkMode::HighLatency => "high latency",
        NetworkMode::PacketLoss => "packet loss",
        NetworkMode::Offline => "offline",
    }
}

// ---------- Benchmarks (PerformanceBenchmarkView.tsx) ----------

pub fn benchmark_view(m: &Model) -> Element<Msg> {
    let b = &m.benchmarks;

    view_shell(
        "High-Performance Engine Diagnostics & Profiler",
        "Live Sub-Millisecond Profiling",
        "Benchmarking memory safety, zero-copy serialization, and GPU-accelerated rendering.",
        vec![
            row()
                .gap(SP4)
                .wrap()
                .children(vec![
                    metric_card(
                        "FTS Search Query Latency",
                        format!("{:.2} ms", b.fts_search_time_ms),
                        ValueColor::Accent,
                        None,
                        "Inverted index tokenization across local message store".into(),
                    ),
                    metric_card(
                        "Storage Insert Throughput",
                        format!("{}", b.insert_throughput_msgs_per_sec),
                        ValueColor::Warning,
                        Some("msgs/sec".into()),
                        "Async pipeline with LZ4 block compression".into(),
                    ),
                    metric_card(
                        "Base Native Memory",
                        format!("{:.1} MB", b.base_memory_mb),
                        ValueColor::Text,
                        Some("-95% vs Electron".into()),
                        "Zero Chromium/V8 engine overhead".into(),
                    ),
                ]),
            // Interactive search test
            card_base()
                .gap(SP3)
                .children([
                    text("Interactive FTS Full-Text Search Benchmark")
                        .size(TextSize::Sm)
                        .weight(Weight::Semibold),
                    {
                        let ti: Element<Msg> = text_input(m.bench_query.clone())
                            .placeholder("Type query to test index lookup time...")
                            .width(320.0)
                            .id("bench-query")
                            .on_input(Msg::BenchQueryInput)
                            .into();
                        row()
                            .gap(SP2)
                            .items_center()
                            .children([ti, action_button("Execute Query", Msg::RunSearchTest, true)])
                    },
                ])
                .child(match &m.search_test_result {
                    Some((count, ms)) => div()
                        .p(SP3)
                        .rounded(R_MD)
                        .themed(|t, s| s.bg(t.accent_bg).border(1.0, t.accent_border))
                        .child(
                            text(format!(
                                "Query matched {count} message(s) • Lookup Latency: {ms} ms"
                            ))
                            .size(TextSize::Xs)
                            .mono()
                            .themed(|t, s| s.color(t.accent_text)),
                        ),
                    None => spacer_w0(),
                })
                .child(spacer_w0()),
            // Frame pacing
            card_base()
                .gap(SP3)
                .child(
                    text("GPU Render Frame Pacing Profile")
                        .size(TextSize::Sm)
                        .weight(Weight::Semibold),
                )
                .child(
                    row()
                        .gap(SP4)
                        .wrap()
                        .children(vec![
                            pacing_box("Render Target:", "120 FPS / 8.33ms".to_string(), ValueColor::Text),
                            pacing_box(
                                "WAL Checkpoint Truncate:",
                                format!("{:.2} ms", b.wal_checkpoint_ms),
                                ValueColor::Accent,
                            ),
                            pacing_box(
                                "LZ4 Block Compression:",
                                "2.4 GB/sec".to_string(),
                                ValueColor::Warning,
                            ),
                        ]),
                ),
        ],
    )
}

fn pacing_box(label: &str, value: String, color: ValueColor) -> Element<Msg> {
    div()
        .p(SP3)
        .rounded(R_MD)
        .min_w(180.0)
        .themed(|t, s| s.bg(t.element))
        .children([
            text(label.to_string())
                .size(TextSize::Xs)
                .themed(|t, s| s.color(t.text_muted)),
            text(value)
                .size(TextSize::Xl)
                .weight(Weight::Semibold)
                .mono()
                .themed(move |t, s| match color {
                    ValueColor::Text => s.color(t.text),
                    ValueColor::Accent => s.color(t.accent_text),
                    ValueColor::Warning => s.color(t.warning.text),
                    ValueColor::Danger => s.color(t.danger.text),
                }),
        ])
}

// ---------- Rust Architecture (RustArchitectureView.tsx) ----------

pub const ARCH_SNIPPETS: [(&str, &str); 4] = [
    ("src/storage/engine.rs", "Rust Lightweight Storage Engine (B-Tree + LZ4 + WAL)"),
    ("src/sync/protocol.rs", "Incremental Sequence Sync Protocol (Protobuf / Delta Sync)"),
    ("src/ui/fenestra_window.rs", "Fenestra Desktop Window & Virtualized List Layout"),
    ("src/sync/worker.rs", "Async Sync Worker & Crossbeam Channel Dispatcher"),
];

pub fn arch_view(m: &Model) -> Element<Msg> {
    let idx = m.arch_snippet_idx.min(ARCH_SNIPPETS.len() - 1);
    let (filename, title) = ARCH_SNIPPETS[idx];
    let code = crate::mock::arch_snippet_code(idx);

    view_shell(
        "Rust + Fenestra Architecture Blueprint",
        "Zero-Cost Systems GUI",
        "Native multi-threading, async sync worker, lockless channels, and memory-safe persistence.",
        vec![
            // Snippet selector tabs
            card_base()
                .gap(SP3)
                .child(
                    row()
                        .gap(SP2)
                        .wrap()
                        .children(
                            ARCH_SNIPPETS
                                .iter()
                                .enumerate()
                                .map(|(i, (_, t))| arch_tab(m, i, t))
                                .collect::<Vec<_>>(),
                        ),
                )
                .child(
                    col()
                        .gap(SP2)
                        .children([
                            text(filename.to_string())
                                .size(TextSize::Xs)
                                .mono()
                                .weight(Weight::Semibold)
                                .themed(|t, s| s.color(t.accent_text)),
                            text(title.to_string())
                                .size(TextSize::Sm)
                                .weight(Weight::Semibold),
                        ]),
                )
                .child(
                    div()
                        .rounded(R_MD)
                        .overflow_hidden()
                        .themed(|t, s| s.bg(t.bg).border(1.0, t.border))
                        .child(
                            text(code)
                                .size(TextSize::Xs)
                                .mono()
                                .p(SP3)
                                .themed(|t, s| s.color(t.success.text)),
                        ),
                ),
            // Comparison table
            comparison_table(),
        ],
    )
}

fn arch_tab(m: &Model, idx: usize, title: &str) -> Element<Msg> {
    let active = m.arch_snippet_idx == idx;
    let base = div()
        .px(SP3)
        .py(SP1)
        .rounded(R_MD)
        .semantics(Semantics::Button)
        .label(format!("Snippet: {title}"))
        .on_click(Msg::SelectArchSnippet(idx));
    if active {
        base.themed(|t, s| s.bg(t.accent)).child(
            text(format!("#{}", idx + 1))
                .size(TextSize::Xs)
                .mono()
                .weight(Weight::Semibold)
                .themed(|t, s| s.color(t.on_accent)),
        )
    } else {
        base.state_layer(|t| t.text).child(
            text(format!("#{}", idx + 1))
                .size(TextSize::Xs)
                .mono()
                .themed(|t, s| s.color(t.text_muted)),
        )
    }
}

struct ComparisonRow {
    metric: &'static str,
    electron: &'static str,
    tauri: &'static str,
    native: &'static str,
}

fn comparison_table() -> Element<Msg> {
    let rows = [
        ComparisonRow {
            metric: "Base RAM Consumption",
            electron: "280 ~ 450 MB",
            tauri: "45 ~ 90 MB",
            native: "14.8 MB (zero V8 overhead)",
        },
        ComparisonRow {
            metric: "Rendering Pipeline",
            electron: "Chromium DOM / Skia",
            tauri: "System Webview",
            native: "Direct GPU tree (120 FPS)",
        },
        ComparisonRow {
            metric: "Cold Startup Time",
            electron: "1,800 ~ 3,200 ms",
            tauri: "350 ~ 600 ms",
            native: "48 ms (instant load)",
        },
        ComparisonRow {
            metric: "Binary Disk Size",
            electron: "120 ~ 180 MB",
            tauri: "12 ~ 25 MB",
            native: "6.4 MB (statically linked)",
        },
        ComparisonRow {
            metric: "GC Stutters",
            electron: "V8 mark-sweep pauses",
            tauri: "Webview GC pauses",
            native: "Zero GC (RAII lifetimes)",
        },
        ComparisonRow {
            metric: "Message Serialization",
            electron: "JSON.stringify / structured clone",
            tauri: "IPC JSON serde",
            native: "Zero-copy rkyv / LZ4 mmap",
        },
    ];

    let header = row()
        .px(SP3)
        .py(SP2)
        .gap(SP3)
        .themed(|t, s| s.border_bottom(1.0, t.border))
        .children([
            text("Metric").size(TextSize::Xs).weight(Weight::Semibold).grow(),
            text("Electron").size(TextSize::Xs).weight(Weight::Semibold).w(140.0),
            text("Tauri").size(TextSize::Xs).weight(Weight::Semibold).w(120.0),
            text("Rust + Fenestra").size(TextSize::Xs).weight(Weight::Semibold).w(200.0),
        ]);

    let body: Vec<Element<Msg>> = rows
        .iter()
        .map(|r| {
            row()
                .px(SP3)
                .py(SP2)
                .gap(SP3)
                .themed(|t, s| s.border_bottom(1.0, t.border_subtle))
                .children([
                    text(r.metric.to_string()).size(TextSize::Xs).grow(),
                    text(r.electron.to_string())
                        .size(TextSize::Xs)
                        .mono()
                        .w(140.0)
                        .themed(|t, s| s.color(t.text_muted)),
                    text(r.tauri.to_string())
                        .size(TextSize::Xs)
                        .mono()
                        .w(120.0)
                        .themed(|t, s| s.color(t.text_muted)),
                    text(r.native.to_string())
                        .size(TextSize::Xs)
                        .mono()
                        .w(200.0)
                        .themed(|t, s| s.color(t.accent_text)),
                ])
        })
        .collect();

    card_base()
        .gap(SP1)
        .child(text("Engine Comparison").size(TextSize::Sm).weight(Weight::Semibold))
        .child(col().child(header).children(body))
}
