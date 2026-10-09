//! Joining the Wi-Fi network a phone hosts for a Quick Share bandwidth upgrade
//! (its Wi-Fi Direct group or hotspot), so a transfer that started over
//! Bluetooth continues at Wi-Fi speed. Dropping the guard leaves the network;
//! Windows then reconnects to the user's usual one.

use std::time::Duration;

use anyhow::anyhow;
use windows::core::HSTRING;
use windows::Devices::WiFi::{
    WiFiAccessStatus, WiFiAdapter, WiFiAvailableNetwork, WiFiConnectionStatus, WiFiReconnectionKind,
};
use windows::Security::Credentials::PasswordCredential;

const INNER_NAME: &str = "WifiJoin";
/// The group is usually created a moment before the credentials reach us.
const SCAN_ATTEMPTS: u32 = 8;

pub struct WifiJoin {
    adapter: WiFiAdapter,
    ssid: String,
}

impl std::fmt::Debug for WifiJoin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WifiJoin").field("ssid", &self.ssid).finish()
    }
}

impl Drop for WifiJoin {
    fn drop(&mut self) {
        let _ = self.adapter.Disconnect();
        info!("{INNER_NAME}: left '{}'", self.ssid);
    }
}

fn find_network(adapter: &WiFiAdapter, ssid: &str) -> Result<Option<WiFiAvailableNetwork>, anyhow::Error> {
    adapter.ScanAsync()?.get()?;
    for net in adapter.NetworkReport()?.AvailableNetworks()? {
        if net.Ssid()?.to_string() == ssid {
            return Ok(Some(net));
        }
    }
    Ok(None)
}

fn join_blocking(ssid: String, password: String) -> Result<WifiJoin, anyhow::Error> {
    if WiFiAdapter::RequestAccessAsync()?.get()? != WiFiAccessStatus::Allowed {
        return Err(anyhow!("Wi-Fi access denied"));
    }
    let adapter = WiFiAdapter::FindAllAdaptersAsync()?
        .get()?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no Wi-Fi adapter"))?;

    let mut network = None;
    for attempt in 1..=SCAN_ATTEMPTS {
        network = find_network(&adapter, &ssid)?;
        if network.is_some() {
            break;
        }
        debug!("{INNER_NAME}: '{ssid}' not visible yet (scan {attempt})");
        std::thread::sleep(Duration::from_millis(800));
    }
    let network = network.ok_or_else(|| anyhow!("network '{ssid}' not found"))?;

    let credential = PasswordCredential::new()?;
    credential.SetPassword(&HSTRING::from(password))?;
    let result = adapter
        .ConnectWithPasswordCredentialAsync(&network, WiFiReconnectionKind::Manual, &credential)?
        .get()?;
    let status = result.ConnectionStatus()?;
    if status != WiFiConnectionStatus::Success {
        return Err(anyhow!("connecting to '{ssid}' failed: {status:?}"));
    }
    info!("{INNER_NAME}: joined '{ssid}'");
    Ok(WifiJoin { adapter, ssid })
}

pub async fn join(ssid: &str, password: &str) -> Result<WifiJoin, anyhow::Error> {
    let (ssid, password) = (ssid.to_owned(), password.to_owned());
    tokio::task::spawn_blocking(move || join_blocking(ssid, password)).await?
}

/// Connects to the phone once DHCP on the new network has given us an address.
pub async fn connect_tcp(host: &str, port: u16) -> Result<tokio::net::TcpStream, anyhow::Error> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        match tokio::time::timeout(Duration::from_secs(3), tokio::net::TcpStream::connect((host, port))).await {
            Ok(Ok(s)) => return Ok(s),
            r if tokio::time::Instant::now() >= deadline => {
                return Err(anyhow!("can't reach {host}:{port}: {r:?}"));
            }
            _ => tokio::time::sleep(Duration::from_millis(500)).await,
        }
    }
}
