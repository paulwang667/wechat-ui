//! Locally vendored Lucide icons beyond fenestra-kit's 24-icon subset.
//! Path data from lucide-static v0.469.0 (ISC license), 24×24 viewbox,
//! stroked at 2px — identical rendering contract to `fenestra_kit::icons::lucide`.

use fenestra::prelude::*;
use kurbo::BezPath;

fn paths<Msg>(ds: &[&str]) -> Element<Msg> {
    let mut bez = BezPath::new();
    for d in ds {
        if let Ok(sub) = BezPath::from_svg(d) {
            for item in sub {
                bez.push(item);
            }
        }
    }
    path(bez, (24.0, 24.0), Some(2.0))
}

/// The extra icon by kebab-case Lucide name; `None` when unknown.
pub fn by_name<Msg>(name: &str) -> Option<Element<Msg>> {
    let ds: &[&str] = match name {
        "message-square" => &["M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"],
        "users" => &[
            "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2",
            "M5 7a4 4 0 1 0 8 0a4 4 0 1 0-8 0", // circle (9,7) r4
            "M22 21v-2a4 4 0 0 0-3-3.87",
            "M16 3.13a4 4 0 0 1 0 7.75",
        ],
        "database" => &[
            "M3 5a9 3 0 1 0 18 0a9 3 0 1 0-18 0", // ellipse (12,5) 9×3
            "M3 5V19A9 3 0 0 0 21 19V5",
            "M3 12A9 3 0 0 0 21 12",
        ],
        "activity" => &["M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2"],
        "code" => &["M16 18L22 12L16 6", "M8 6L2 12L8 18"],
        "gauge" => &["m12 14 4-4", "M3.34 19a10 10 0 1 1 17.32 0"],
        "zap" => &["M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z"],
        "bell-off" => &[
            "M10.268 21a2 2 0 0 0 3.464 0",
            "M17 17H4a1 1 0 0 1-.74-1.673C4.59 13.956 6 12.499 6 8a6 6 0 0 1 .258-1.742",
            "m2 2 20 20",
            "M8.668 3.01A6 6 0 0 1 18 8c0 2.687.77 4.653 1.707 6.05",
        ],
        _ => return None,
    };
    Some(paths(ds))
}
