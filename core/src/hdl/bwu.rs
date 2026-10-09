//! Bandwidth upgrade (BWU) pieces shared by sending and receiving: a transfer
//! that started over Bluetooth moves to a Wi-Fi network the phone hosts (its
//! Wi-Fi Direct group or hotspot) or to its LAN address. We are always the
//! client: we join and connect, the phone hosts and listens.
//! Frame sequence from open-quickshare (GPL-3.0) and google/nearby bwu_manager.

use std::time::Duration;

use anyhow::anyhow;
use prost::Message;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::location_nearby_connections::bandwidth_upgrade_negotiation_frame::upgrade_path_info::Medium;
use crate::location_nearby_connections::bandwidth_upgrade_negotiation_frame::{
    ClientIntroduction, EventType, UpgradePathInfo,
};
use crate::location_nearby_connections::{
    self as lnc, BandwidthUpgradeNegotiationFrame, MediumMetadata, MediumRole, OfflineFrame,
};
use crate::utils::stream_read_exact;

/// Payloads above this move to Wi-Fi: joining takes a few seconds, which is
/// how long Bluetooth needs for about a megabyte.
pub const MIN_UPGRADE_BYTES: u64 = 1024 * 1024;

/// Mediums we can be upgraded to, in the order we prefer them.
pub fn mediums() -> Vec<i32> {
    vec![
        Medium::WifiLan.into(),
        Medium::WifiDirect.into(),
        Medium::WifiHotspot.into(),
    ]
}

/// Client only: the phone hosts, we join (hosting from Windows is not implemented).
pub fn client_metadata() -> MediumMetadata {
    MediumMetadata {
        supports_5_ghz: Some(true),
        medium_role: Some(MediumRole {
            support_wifi_direct_group_client: Some(true),
            support_wifi_hotspot_client: Some(true),
            ..Default::default()
        }),
        supported_wifi_direct_auth_types: vec![
            lnc::medium_metadata::WifiDirectAuthType::WifiDirectWithPassword.into(),
        ],
        ..Default::default()
    }
}

pub fn frame(
    event: EventType,
    upgrade_path_info: Option<UpgradePathInfo>,
    client_introduction: Option<ClientIntroduction>,
) -> OfflineFrame {
    OfflineFrame {
        version: Some(lnc::offline_frame::Version::V1.into()),
        v1: Some(lnc::V1Frame {
            r#type: Some(lnc::v1_frame::FrameType::BandwidthUpgradeNegotiation.into()),
            bandwidth_upgrade_negotiation: Some(BandwidthUpgradeNegotiationFrame {
                event_type: Some(event.into()),
                upgrade_path_info,
                client_introduction,
                ..Default::default()
            }),
            ..Default::default()
        }),
    }
}

/// Receiver side: ask the phone to host Wi-Fi Direct (the "role switch").
pub fn path_request() -> OfflineFrame {
    frame(
        EventType::UpgradePathRequest,
        Some(UpgradePathInfo {
            medium: Some(Medium::WifiDirect.into()),
            upgrade_path_request: Some(
                lnc::bandwidth_upgrade_negotiation_frame::upgrade_path_info::UpgradePathRequest {
                    mediums: mediums(),
                    medium_meta_data: Some(client_metadata()),
                },
            ),
            ..Default::default()
        }),
        None,
    )
}

pub fn failure(medium: Option<i32>) -> OfflineFrame {
    frame(
        EventType::UpgradeFailure,
        Some(UpgradePathInfo { medium, ..Default::default() }),
        None,
    )
}

/// The upgrade frame inside an offline frame, if it is one.
pub fn event(offline: &OfflineFrame) -> Option<&BandwidthUpgradeNegotiationFrame> {
    offline.v1.as_ref()?.bandwidth_upgrade_negotiation.as_ref()
}

/// A Wi-Fi channel the phone accepted us on, plus the network membership
/// (if we had to join one) that must outlive the transfer.
pub struct Upgraded {
    pub tcp: TcpStream,
    #[cfg(windows)]
    pub wifi: Option<super::wifi_join::WifiJoin>,
}

/// Reaches the network in an UPGRADE_PATH_AVAILABLE offer and introduces us
/// on the new channel (plaintext, as Nearby does before switching).
pub async fn reach(upi: &UpgradePathInfo, endpoint_id: &str) -> Result<Upgraded, anyhow::Error> {
    #[cfg(windows)]
    let mut wifi = None;
    let mut tcp = match upi.medium() {
        Medium::WifiLan => {
            let s = upi.wifi_lan_socket.as_ref().ok_or_else(|| anyhow!("no LAN socket"))?;
            let ip: [u8; 4] = s.ip_address().try_into().map_err(|_| anyhow!("bad LAN address"))?;
            let addr = std::net::Ipv4Addr::from(ip).to_string();
            info!("BWU: phone offers its LAN address {addr}:{}", s.wifi_port());
            TcpStream::connect((addr.as_str(), s.wifi_port() as u16)).await?
        }
        #[cfg(windows)]
        Medium::WifiDirect | Medium::WifiHotspot => {
            let (ssid, password, gateway, port) = if let Some(c) = upi.wifi_direct_credentials.as_ref() {
                (c.ssid().to_owned(), c.password().to_owned(), c.gateway().to_owned(), c.port())
            } else if let Some(c) = upi.wifi_hotspot_credentials.as_ref() {
                (c.ssid().to_owned(), c.password().to_owned(), c.gateway().to_owned(), c.port())
            } else {
                return Err(anyhow!("offer without credentials"));
            };
            info!("BWU: phone hosts {:?} '{ssid}', gateway {gateway}:{port}; joining", upi.medium());
            wifi = Some(super::wifi_join::join(&ssid, &password).await?);
            super::wifi_join::connect_tcp(&gateway, port as u16).await?
        }
        other => return Err(anyhow!("unsupported medium {other:?}")),
    };

    let intro = frame(
        EventType::ClientIntroduction,
        None,
        Some(ClientIntroduction {
            endpoint_id: Some(endpoint_id.to_owned()),
            supports_disabling_encryption: Some(false),
        }),
    )
    .encode_to_vec();
    tcp.write_all(&(intro.len() as u32).to_be_bytes()).await?;
    tcp.write_all(&intro).await?;
    tcp.flush().await?;
    // The ack is optional: read it if it comes, ignore its content.
    let _ = tokio::time::timeout(Duration::from_secs(3), async {
        let mut len = [0u8; 4];
        stream_read_exact(&mut tcp, &mut len).await?;
        let mut body = vec![0u8; (u32::from_be_bytes(len) as usize).min(64 * 1024)];
        stream_read_exact(&mut tcp, &mut body).await
    })
    .await;

    Ok(Upgraded {
        tcp,
        #[cfg(windows)]
        wifi,
    })
}
