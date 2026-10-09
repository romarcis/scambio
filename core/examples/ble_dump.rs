//! Diagnostic: dumps every BLE advertisement carrying service data or 16-bit
//! service UUIDs for 30 s, plus the adapter's extended-advertising support.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use windows::Devices::Bluetooth::Advertisement::*;
use windows::Devices::Bluetooth::BluetoothAdapter;
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::DataReader;

fn main() -> anyhow::Result<()> {
    let adapter = BluetoothAdapter::GetDefaultAsync()?.get()?;
    println!(
        "adapter: LE={} central={} extended_adv={} classic={}",
        adapter.IsLowEnergySupported()?,
        adapter.IsCentralRoleSupported()?,
        adapter.IsExtendedAdvertisingSupported().unwrap_or(false),
        adapter.IsClassicSupported()?
    );

    let seen: Arc<Mutex<BTreeMap<String, u32>>> = Default::default();
    let total = Arc::new(Mutex::new(0u32));
    let watcher = BluetoothLEAdvertisementWatcher::new()?;
    watcher.SetScanningMode(BluetoothLEScanningMode::Active)?;
    println!("allow extended: {:?}", watcher.SetAllowExtendedAdvertisements(true));
    let (s, t) = (seen.clone(), total.clone());
    watcher.Received(&TypedEventHandler::new(
        move |_: &Option<BluetoothLEAdvertisementWatcher>, a: &Option<BluetoothLEAdvertisementReceivedEventArgs>| {
            let a = a.as_ref().unwrap();
            *t.lock().unwrap() += 1;
            let adv = a.Advertisement()?;
            let mut parts = Vec::new();
            for sec in adv.DataSections()? {
                let ty = sec.DataType()?;
                if !matches!(ty, 0x02 | 0x03 | 0x16 | 0xFF) {
                    continue;
                }
                let buf = sec.Data()?;
                let mut b = vec![0u8; buf.Length()? as usize];
                DataReader::FromBuffer(&buf)?.ReadBytes(&mut b)?;
                parts.push(format!("{ty:02X}:{}", hex::encode(&b)));
            }
            if !parts.is_empty() {
                let key = format!("{:012X} ext={} {}", a.BluetoothAddress()?, a.IsScannable().unwrap_or(false), parts.join(" "));
                *s.lock().unwrap().entry(key).or_default() += 1;
            }
            Ok(())
        },
    ))?;
    watcher.Start()?;
    std::thread::sleep(std::time::Duration::from_secs(30));
    watcher.Stop()?;
    println!("total advertisements: {}", total.lock().unwrap());
    for (k, n) in seen.lock().unwrap().iter() {
        let mark = if k.contains("f3fe") || k.contains("2cfe") { ">>" } else { "  " };
        println!("{mark} x{n:<4} {k}");
    }
    Ok(())
}
