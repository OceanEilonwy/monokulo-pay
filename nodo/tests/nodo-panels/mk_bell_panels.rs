//! The Bell's shipped panel, run on data shaped like what its service returns.
//!
//! Copied into monokulo-nodo's crates/nodo-panels/tests by its app-release workflow, beside the
//! pinned recipe in packages/, so the paths below resolve there.

use nodo_manifest::{Manifest, PanelSource};
use nodo_panels::{run_panel, Status};
use serde_json::json;

const BELL_MANIFEST: &str = include_str!("../../../packages/mk-bell/files/manifest.toml");
const BELL_PANEL: &str = include_str!("../../../packages/mk-bell/files/panels/bell.lua");

/// monokulo-bell's GET /status, as crates/monokulo-bell/tests/bell_over_http.rs shows it.
fn bell_status(stored: Option<u64>, clover_na: &str, last: Option<&str>) -> serde_json::Value {
    json!({
        "service": "monokulo-bell", "version": "0.1.0", "commit": null, "started_at": 1_794_000_000,
        "listen": "127.0.0.1:8445", "public_uri": "https://bell.example.org",
        "registrations": { "stored": stored },
        "points_of_sale": {
            "square": { "production": "ready", "sandbox": "off" },
            "clover": { "north_america": clover_na, "europe": "off", "latin_america": "off", "sandbox": "off" },
        },
        "last_ding": last.map(|outcome| json!({ "at": 1_794_000_100, "outcome": outcome })),
    })
}

#[test]
fn the_bells_manifest_and_panel() {
    let manifest = Manifest::parse(&BELL_MANIFEST.replace("@VERSION@", "0.1.0")).unwrap();
    assert_eq!(manifest.panels[0].source, PanelSource::Http { url: "http://127.0.0.1:8447/status".into() });

    let out = run_panel(BELL_PANEL, "bell/bell", &bell_status(Some(3), "ready", Some("sent")), &[]).unwrap();
    assert_eq!(out.big.as_deref(), Some("3"));
    assert_eq!(out.unit.as_deref(), Some("tills"));
    assert_eq!(out.status, Some(Status::Ok));
    assert!(out.rows.contains(&("serving".into(), "Square, Clover NA".into())), "{:?}", out.rows);
    assert!(out.rows.contains(&("last ding".into(), "sent".into())));

    let fresh = run_panel(BELL_PANEL, "bell/bell", &bell_status(Some(0), "off", None), &[]).unwrap();
    assert_eq!(fresh.status, Some(Status::Idle));
    assert!(fresh.rows.contains(&("serving".into(), "Square".into())));

    let failed = run_panel(BELL_PANEL, "bell/bell", &bell_status(Some(1), "off", Some("failed")), &[]).unwrap();
    assert_eq!(failed.unit.as_deref(), Some("till"));
    assert_eq!(failed.status, Some(Status::Warn));

    let mut all = bell_status(None, "ready", None);
    all["points_of_sale"] = json!({
        "square": { "production": "ready", "sandbox": "ready" },
        "clover": { "north_america": "ready", "europe": "ready", "latin_america": "ready", "sandbox": "ready" },
    });
    let all = run_panel(BELL_PANEL, "bell/bell", &all, &[]).unwrap();
    assert!(all.rows.contains(&("serving".into(), "6 points of sale".into())), "{:?}", all.rows);
    assert_eq!(all.big.as_deref(), Some("--"));
}
