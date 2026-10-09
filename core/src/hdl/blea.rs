use tokio_util::sync::CancellationToken;
use windows::Devices::Bluetooth::Advertisement::{
    BluetoothLEAdvertisementDataSection, BluetoothLEAdvertisementPublisher,
};
use windows::Storage::Streams::DataWriter;

// Quick Share "a device nearby wants to share" beacon: makes Android phones
// start advertising themselves over mDNS so discovery can find them.
const SERVICE_DATA: [u8; 24] = [
    252, 18, 142, 1, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 191, 45, 91, 160, 225, 216, 117, 36, 202, 0,
];

const INNER_NAME: &str = "BleAdvertiser";

pub struct BleAdvertiser {
    publisher: BluetoothLEAdvertisementPublisher,
}

impl BleAdvertiser {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let publisher = BluetoothLEAdvertisementPublisher::new()?;

        let writer = DataWriter::new()?;
        // 16-bit service UUID 0xFE2C (little endian) followed by its data
        writer.WriteBytes(&[0x2C, 0xFE])?;
        writer.WriteBytes(&SERVICE_DATA)?;

        let section = BluetoothLEAdvertisementDataSection::new()?;
        section.SetDataType(0x16)?; // Service Data - 16-bit UUID
        section.SetData(&writer.DetachBuffer()?)?;
        publisher.Advertisement()?.DataSections()?.Append(&section)?;

        Ok(Self { publisher })
    }

    pub async fn run(&self, ctk: CancellationToken) -> Result<(), anyhow::Error> {
        self.publisher.Start()?;
        info!("{INNER_NAME}: advertising");
        ctk.cancelled().await;
        let _ = self.publisher.Stop();
        info!("{INNER_NAME}: tracker cancelled, returning");

        Ok(())
    }
}

/// This PC as a Quick Share receiver over BLE (phone -> PC, no shared network):
/// the same 0xFEF3 layout phones use, carrying our Bluetooth MAC so the phone
/// dials our RFCOMM listener. Too long for a legacy PDU: extended advertising.
pub struct ReceiverAdvertiser {
    publisher: BluetoothLEAdvertisementPublisher,
}

/// `[0x48][svc hash 3][u32 len][data][device token 2][extra fields 0x00]` with
/// `data = [0x23][svc hash 3][endpoint_id 4][einfo len][einfo][mac 6][extra 2]`.
pub fn receiver_service_data(endpoint_id: [u8; 4], einfo: &[u8], mac: [u8; 6]) -> Vec<u8> {
    const SVC_HASH: [u8; 3] = [0xFC, 0x9F, 0x5E];
    let mut data = vec![0x23];
    data.extend_from_slice(&SVC_HASH);
    data.extend_from_slice(&endpoint_id);
    data.push(einfo.len() as u8);
    data.extend_from_slice(einfo);
    data.extend_from_slice(&mac);
    data.extend_from_slice(&[0, 0]);

    let mut sd = vec![0x48];
    sd.extend_from_slice(&SVC_HASH);
    sd.extend_from_slice(&(data.len() as u32).to_be_bytes());
    sd.extend_from_slice(&data);
    sd.extend_from_slice(&rand::random::<[u8; 2]>());
    sd.push(0x00);
    sd
}

impl ReceiverAdvertiser {
    pub fn new(service_data: &[u8]) -> Result<Self, anyhow::Error> {
        let publisher = BluetoothLEAdvertisementPublisher::new()?;
        publisher.SetUseExtendedAdvertisement(true)?;

        let writer = DataWriter::new()?;
        writer.WriteBytes(&[0xF3, 0xFE])?;
        writer.WriteBytes(service_data)?;
        let section = BluetoothLEAdvertisementDataSection::new()?;
        section.SetDataType(0x16)?;
        section.SetData(&writer.DetachBuffer()?)?;
        publisher.Advertisement()?.DataSections()?.Append(&section)?;
        Ok(Self { publisher })
    }

    /// Advertises while `visibility` is not Invisible, until `ctk` is cancelled.
    pub async fn run(
        &self,
        mut visibility: tokio::sync::watch::Receiver<crate::Visibility>,
        ctk: CancellationToken,
    ) -> Result<(), anyhow::Error> {
        let mut was_on = false;
        loop {
            let on = *visibility.borrow_and_update() != crate::Visibility::Invisible;
            if on == was_on {
            } else if on {
                self.publisher.Start()?;
                info!("ReceiverAdvertiser: advertising");
            } else {
                let _ = self.publisher.Stop();
                info!("ReceiverAdvertiser: stopped (invisible)");
            }
            was_on = on;
            tokio::select! {
                _ = ctk.cancelled() => break,
                r = visibility.changed() => if r.is_err() { break },
            }
        }
        let _ = self.publisher.Stop();
        Ok(())
    }
}
