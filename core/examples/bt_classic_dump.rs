//! Diagnostic: lists unpaired Bluetooth Classic devices found by an inquiry,
//! decoding Nearby Connections names (base64 of [ver|pcp][endpoint 4][svc hash 3]...).
use base64::Engine;
use windows::Devices::Bluetooth::BluetoothDevice;
use windows::Devices::Enumeration::DeviceInformation;

fn main() -> anyhow::Result<()> {
    let b64 = |s: &str| {
        let t = s.trim_end_matches('=');
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(t)
            .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(t))
            .ok()
    };
    for round in 1..=3 {
        let selector = BluetoothDevice::GetDeviceSelectorFromPairingState(false)?;
        let list = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.get()?;
        println!("round {round}: {} devices", list.Size()?);
        for d in list {
            let name = d.Name()?.to_string();
            let addr = BluetoothDevice::FromIdAsync(&d.Id()?)
                .and_then(|op| op.get())
                .and_then(|b| b.BluetoothAddress())
                .map(|a| format!("{a:012X}"))
                .unwrap_or_else(|_| "?".into());
            let nearby = b64(&name)
                .filter(|b| b.len() >= 8 && b[0] >> 5 == 1)
                .map(|b| format!("  NEARBY pcp={} endpoint={} svc={}", b[0] & 0x1f, String::from_utf8_lossy(&b[1..5]), hex::encode(&b[5..8])))
                .unwrap_or_default();
            println!("  {addr} {name:?}{nearby}");
        }
    }
    Ok(())
}
