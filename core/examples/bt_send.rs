//! Diagnostic: finds a phone in Quick Share receive mode over BLE and sends it
//! a small text file over Bluetooth RFCOMM, printing every state change.
//! Run: cargo run -p rqs_lib --example bt_send [path]
use std::time::Duration;

use rqs_lib::channel::ChannelDirection;
use rqs_lib::{OutboundPayload, SendInfo, Visibility, RQS};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info,rqs_lib=debug").init();
    let file = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            let p = std::env::temp_dir().join("scambio-test.txt");
            std::fs::write(&p, "Ciao dal PC, via Bluetooth!\n")?;
            p.display().to_string()
        }
    };

    let (tx, mut rx) = mpsc::channel(64);
    let ctk = CancellationToken::new();
    tokio::spawn(rqs_lib::ble_scan::scan(tx, ctk.clone()));
    let target = tokio::time::timeout(Duration::from_secs(30), async {
        while let Some(r) = rx.recv().await {
            if r.bt_mac.is_some() && r.visible {
                return Some(r);
            }
        }
        None
    })
    .await
    .ok()
    .flatten()
    .ok_or_else(|| anyhow::anyhow!("no visible receiver with a Bluetooth MAC in 30 s"))?;
    ctk.cancel();
    let addr = rqs_lib::rfcomm::format_bt_addr(target.bt_mac.unwrap());
    println!(">> target {:?} at {addr}", target.name);

    let mut rqs = RQS::new(Visibility::Invisible, None, None);
    let (send, _) = rqs.run().await?;
    let mut msgs = rqs.message_sender.subscribe();
    send.send(SendInfo {
        id: addr.clone(),
        name: target.name.clone().unwrap_or_default(),
        addr,
        ob: OutboundPayload::Files(vec![file]),
    })
    .await?;

    loop {
        let m = tokio::time::timeout(Duration::from_secs(120), msgs.recv()).await??;
        if m.direction != ChannelDirection::LibToFront {
            continue;
        }
        let meta = m.meta.as_ref();
        println!(
            ">> state={:?} pin={:?} {}/{}",
            m.state,
            meta.and_then(|x| x.pin_code.clone()),
            meta.map(|x| x.ack_bytes).unwrap_or(0),
            meta.map(|x| x.total_bytes).unwrap_or(0)
        );
        use rqs_lib::State::*;
        if matches!(m.state, Some(Finished | Rejected | Cancelled | Disconnected)) {
            break;
        }
    }
    rqs.stop().await;
    Ok(())
}
