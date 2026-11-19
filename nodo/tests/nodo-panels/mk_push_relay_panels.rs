//! The push relay's shipped panel, run on data shaped like what its service returns.
//!
//! Copied into monokulo-nodo's crates/nodo-panels/tests by its app-release workflow, beside the
//! pinned recipe in packages/, so the paths below resolve there.

use nodo_manifest::{Manifest, PanelSource};
use nodo_panels::{run_panel, Status};
use serde_json::json;

const PUSH_RELAY_MANIFEST: &str = include_str!("../../../packages/mk-push-relay/files/manifest.toml");
const PUSH_RELAY_PANEL: &str = include_str!("../../../packages/mk-push-relay/files/panels/relay.lua");

/// monokulo-push-relay's GET /status, as crates/push-relay/tests/relay_over_http.rs shows it.
fn relay_status(stored: Option<u64>, apns: &str, fcm: &str, last: Option<&str>) -> serde_json::Value {
    json!({
        "service": "monokulo-push-relay", "version": "0.1.0", "commit": "4e3c6042c312b68020e27f22fdb8d954e5df4371",
        "started_at": 1_794_000_000, "listen": "127.0.0.1:8444", "public_uri": "https://push.example.org",
        "apps": ["monokular"], "handles": { "stored": stored },
        "platforms": { "apns": apns, "fcm": fcm },
        "last_delivery": last.map(|outcome| json!({ "at": 1_794_000_100, "outcome": outcome })),
    })
}

#[test]
fn the_push_relays_manifest_and_panel() {
    let manifest = Manifest::parse(&PUSH_RELAY_MANIFEST.replace("@VERSION@", "0.1.0")).unwrap();
    assert_eq!(manifest.panels[0].source, PanelSource::Http { url: "http://127.0.0.1:8446/status".into() });

    let out = run_panel(PUSH_RELAY_PANEL, "push-relay/relay", &relay_status(Some(12), "ready", "ready", Some("sent")), &[]).unwrap();
    assert_eq!(out.big.as_deref(), Some("12"));
    assert_eq!(out.unit.as_deref(), Some("handles"));
    assert_eq!(out.status, Some(Status::Ok));
    assert!(out.rows.contains(&("last push".into(), "sent".into())), "{:?}", out.rows);

    let fresh = run_panel(PUSH_RELAY_PANEL, "push-relay/relay", &relay_status(Some(1), "ready", "not-configured", None), &[]).unwrap();
    assert_eq!(fresh.unit.as_deref(), Some("handle"));
    assert!(fresh.rows.contains(&("FCM".into(), "not set up".into())));
    assert_eq!(fresh.status, Some(Status::Ok));

    let bare = run_panel(PUSH_RELAY_PANEL, "push-relay/relay", &relay_status(Some(0), "not-configured", "not-configured", None), &[]).unwrap();
    assert_eq!(bare.status, Some(Status::Warn));
    assert_eq!(bare.note.as_deref(), Some("no APNs or FCM credentials set"));

    let busy = run_panel(PUSH_RELAY_PANEL, "push-relay/relay", &relay_status(None, "ready", "ready", Some("unavailable")), &[]).unwrap();
    assert_eq!(busy.big.as_deref(), Some("--"));
    assert_eq!(busy.status, Some(Status::Warn));
}
