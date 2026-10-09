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
