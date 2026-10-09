//! Diagnostic: lists phones in Quick Share receive mode seen over BLE.
//! Run: cargo run -p rqs_lib --example ble_probe   (Ctrl+C to stop after 30 s)
use std::collections::HashSet;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info,rqs_lib=debug").init();
    let (tx, mut rx) = mpsc::channel(64);
    let ctk = CancellationToken::new();
    let scan = tokio::spawn(rqs_lib::ble_scan::scan(tx, ctk.clone()));
    let mut seen = HashSet::new();
    let deadline = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = &mut deadline => break,
            Some(r) = rx.recv() => {
                if seen.insert((r.ble_address, r.raw.clone())) {
                    println!(
                        "addr={:012X} rssi={} name={:?} type={:?} visible={} endpoint={} bt_mac={} psm={:?}\n  raw={}",
                        r.ble_address, r.rssi, r.name, r.device_type, r.visible,
                        String::from_utf8_lossy(&r.endpoint_id),
                        r.bt_mac.map(|m| m.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(":")).unwrap_or_else(|| "-".into()),
                        r.psm, hex::encode(&r.raw)
                    );
                }
            }
        }
    }
    ctk.cancel();
    let _ = scan.await;
    println!("done: {} distinct advertisements", seen.len());
    Ok(())
}
