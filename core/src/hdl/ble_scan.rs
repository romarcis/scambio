//! BLE discovery of Android phones in Quick Share receive mode (PC -> phone,
//! no shared network). A receiving phone advertises a Nearby Connections
//! endpoint under service 0xFEF3; the full ("Everyone") form is an *extended*
//! advertisement, so the Windows watcher must allow extended advertisements.
//!
//! Advertisement decoding follows open-quickshare's `ble_client.rs`
//! (GPL-3.0, https://github.com/ignotusbucius/open-quickshare).

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use windows::Devices::Bluetooth::Advertisement::{
    BluetoothLEAdvertisementReceivedEventArgs, BluetoothLEAdvertisementWatcher,
    BluetoothLEScanningMode,
};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::DataReader;

use crate::utils::DeviceType;

const INNER_NAME: &str = "BleScan";

/// SHA-256("NearbySharing")[0..3]
pub const SVC_HASH: [u8; 3] = [0xFC, 0x9F, 0x5E];
/// 16-bit service UUID 0xFEF3, little endian as it appears in service data.
const QS_SERVICE_UUID_LE: [u8; 2] = [0xF3, 0xFE];

/// A phone advertising itself as a Quick Share receiver over BLE.
#[derive(Debug, Clone)]
pub struct BleReceiver {
    /// Bluetooth LE address the advertisement came from (may rotate).
    pub ble_address: u64,
    pub rssi: i16,
    pub endpoint_id: [u8; 4],
    pub name: Option<String>,
    pub device_type: DeviceType,
    /// endpoint_info visibility bit clear == visible to "Everyone".
    pub visible: bool,
    /// Bluetooth Classic MAC carried by the full advertisement (RFCOMM target).
    pub bt_mac: Option<[u8; 6]>,
    /// LE L2CAP PSM, when advertised (not usable from Windows).
    pub psm: Option<u16>,
    /// The raw service data, kept for diagnostics.
    pub raw: Vec<u8>,
}

/// Decodes the 0xFEF3 service data of a receiving phone.
///
/// * full `[0x48][svc hash 3][u32 len][data][token 2][mask][psm 2]?`
/// * fast `[0x4a][u8 len][data][token 2]` (background beacon: no name, no MAC)
///
/// `data` is `[0x23][svc hash 3]?[endpoint_id 4][einfo len][einfo][mac 6][extra 2]?`
/// (service hash and MAC only in the full form).
pub fn decode_receiver_advert(sd: &[u8]) -> Option<BleReceiver> {
    if sd.is_empty() || sd[0] >> 5 != 2 {
        return None;
    }
    let fast = sd[0] & 0x02 != 0;
    let (data, trailer): (&[u8], &[u8]) = if fast {
        let len = *sd.get(1)? as usize;
        (sd.get(2..2 + len)?, sd.get(2 + len..)?)
    } else {
        if sd.len() < 8 || sd[1..4] != SVC_HASH {
            return None;
        }
        let len = u32::from_be_bytes([sd[4], sd[5], sd[6], sd[7]]) as usize;
        (sd.get(8..8 + len)?, sd.get(8 + len..)?)
    };

    let eid_at = if fast { 1 } else { 4 };
    if !fast && (data.len() < 4 || data[1..4] != SVC_HASH) {
        return None;
    }
    let endpoint_id: [u8; 4] = data.get(eid_at..eid_at + 4)?.try_into().ok()?;

    let elen = *data.get(eid_at + 4)? as usize;
    let einfo_at = eid_at + 5;
    let einfo = data.get(einfo_at..einfo_at + elen)?;
    // einfo: [ver 3b|visibility 1b|device_type 3b|reserved][identity 16][name len][name]
    let (device_type, visible) = match einfo.first() {
        Some(b) => (DeviceType::from_raw_value((b >> 1) & 0x7), b & 0x10 == 0),
        None => (DeviceType::Unknown, true),
    };
    let name = einfo.get(17).and_then(|nlen| {
        einfo
            .get(18..18 + *nlen as usize)
            .map(|n| String::from_utf8_lossy(n).into_owned())
    });

    let bt_mac = if fast {
        None
    } else {
        data.get(einfo_at + elen..einfo_at + elen + 6)
            .and_then(|m| <[u8; 6]>::try_from(m).ok())
            .filter(|m| m.iter().any(|b| *b != 0))
    };

    let psm = if !fast && trailer.len() >= 5 && trailer[2] & 0x01 != 0 {
        Some(u16::from_be_bytes([trailer[3], trailer[4]])).filter(|p| *p != 0)
    } else {
        None
    };

    Some(BleReceiver {
        ble_address: 0,
        rssi: 0,
        endpoint_id,
        name,
        device_type,
        visible,
        bt_mac,
        psm,
        raw: sd.to_vec(),
    })
}

/// Pulls every 0xFEF3 service-data payload out of one received advertisement.
fn fef3_service_data(args: &BluetoothLEAdvertisementReceivedEventArgs) -> windows::core::Result<Vec<Vec<u8>>> {
    let mut out = Vec::new();
    for section in args.Advertisement()?.DataSections()? {
        // 0x16 = Service Data, 16-bit UUID
        if section.DataType()? != 0x16 {
            continue;
        }
        let buf = section.Data()?;
        let mut bytes = vec![0u8; buf.Length()? as usize];
        DataReader::FromBuffer(&buf)?.ReadBytes(&mut bytes)?;
        if bytes.len() > 2 && bytes[..2] == QS_SERVICE_UUID_LE {
            out.push(bytes[2..].to_vec());
        }
    }
    Ok(out)
}

/// Scans until `ctk` is cancelled, sending every decoded receiver advertisement.
/// Undecodable 0xFEF3 payloads are logged so new layouts can be diagnosed.
pub async fn scan(tx: mpsc::Sender<BleReceiver>, ctk: CancellationToken) -> Result<(), anyhow::Error> {
    let watcher = BluetoothLEAdvertisementWatcher::new()?;
    watcher.SetScanningMode(BluetoothLEScanningMode::Active)?;
    // Windows 10 2004+: the full receiver advertisement does not fit a legacy PDU.
    if let Err(e) = watcher.SetAllowExtendedAdvertisements(true) {
        warn!("{INNER_NAME}: extended advertisements unavailable: {e}");
    }

    // The handler is a temporary: it must not live across the await below (not Send).
    let token = watcher.Received(&TypedEventHandler::new(
        move |_: &Option<BluetoothLEAdvertisementWatcher>,
              args: &Option<BluetoothLEAdvertisementReceivedEventArgs>| {
            let Some(args) = args else { return Ok(()) };
            for sd in fef3_service_data(args)? {
                match decode_receiver_advert(&sd) {
                    Some(mut r) => {
                        r.ble_address = args.BluetoothAddress()?;
                        r.rssi = args.RawSignalStrengthInDBm()?;
                        let _ = tx.try_send(r);
                    }
                    None => debug!("{INNER_NAME}: undecoded FEF3 data {}", hex::encode(&sd)),
                }
            }
            Ok(())
        },
    ))?;
    watcher.Start()?;
    info!("{INNER_NAME}: scanning for Quick Share receivers");

    ctk.cancelled().await;
    let _ = watcher.Stop();
    let _ = watcher.RemoveReceived(token);
    info!("{INNER_NAME}: stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_full_advert_with_name_and_mac() {
        let name = b"Pixel";
        let mut einfo = vec![0x22]; // ver 1, visible, device_type phone(1)
        einfo.extend([0u8; 16]);
        einfo.push(name.len() as u8);
        einfo.extend(name);
        let mut data = vec![0x23];
        data.extend(SVC_HASH);
        data.extend(*b"ABCD");
        data.push(einfo.len() as u8);
        data.extend(&einfo);
        data.extend([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        let mut sd = vec![0x48];
        sd.extend(SVC_HASH);
        sd.extend((data.len() as u32).to_be_bytes());
        sd.extend(&data);
        sd.extend([0xAA, 0xBB, 0x01, 0x00, 0x81]);

        let r = decode_receiver_advert(&sd).unwrap();
        assert_eq!(&r.endpoint_id, b"ABCD");
        assert_eq!(r.name.as_deref(), Some("Pixel"));
        assert_eq!(r.device_type, DeviceType::Phone);
        assert!(r.visible);
        assert_eq!(r.bt_mac, Some([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]));
        assert_eq!(r.psm, Some(0x81));
    }

    #[test]
    fn rejects_other_service_data() {
        assert!(decode_receiver_advert(&[0xFC, 0x12, 0x8E]).is_none());
    }
}

#[cfg(test)]
mod real_capture {
    use super::*;

    /// Captured from a OnePlus 12 on its Quick Share receive screen (Wi-Fi on, not connected).
    #[test]
    fn decodes_oneplus_receive_screen() {
        let sd = hex::decode("48fc9f5e0000003823fc9f5e30534435272288dbdf5698522e596e81631991f66ba715736d61727470686f6e6520646920526f626572746facc0486495a20000782303008c39a017e53658529ca3af2fa583279482a6318c1597d09f7de1b2cb24ecdebca7e42f148415057f1f00841506211f008415047f1f00831501ffff").unwrap();
        let r = decode_receiver_advert(&sd).unwrap();
        assert_eq!(&r.endpoint_id, b"0SD5");
        assert_eq!(r.name.as_deref(), Some("smartphone di Roberto"));
        assert_eq!(r.device_type, DeviceType::Phone);
        assert!(r.visible);
        assert_eq!(r.bt_mac, Some([0xAC, 0xC0, 0x48, 0x64, 0x95, 0xA2]));
    }
}
