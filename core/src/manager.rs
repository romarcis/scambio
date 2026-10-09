use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::channel::{ChannelDirection, ChannelMessage};
use crate::errors::AppError;
use crate::hdl::{InboundRequest, OutboundPayload, OutboundRequest, State};
use crate::utils::RemoteDeviceInfo;

const INNER_NAME: &str = "TcpServer";

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct SendInfo {
    pub id: String,
    pub name: String,
    pub addr: String,
    pub ob: OutboundPayload,
}

pub struct TcpServer {
    endpoint_id: [u8; 4],
    tcp_listener: TcpListener,
    sender: Sender<ChannelMessage>,
    connect_receiver: Receiver<SendInfo>,
}

impl TcpServer {
    pub fn new(
        endpoint_id: [u8; 4],
        tcp_listener: TcpListener,
        sender: Sender<ChannelMessage>,
        connect_receiver: Receiver<SendInfo>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            endpoint_id,
            tcp_listener,
            sender,
            connect_receiver,
        })
    }

    pub async fn run(&mut self, ctk: CancellationToken) -> Result<(), anyhow::Error> {
        info!("{INNER_NAME}: service starting");

        loop {
            let cctk = ctk.clone();

            tokio::select! {
                _ = ctk.cancelled() => {
                    info!("{INNER_NAME}: tracker cancelled, breaking");
                    break;
                }
                Some(i) = self.connect_receiver.recv() => {
                    info!("{INNER_NAME}: connect_receiver: got {:?}", i);
                    if let Err(e) = self.connect(cctk, i).await {
                        error!("{INNER_NAME}: error sending: {}", e.to_string());
                    }
                }
                r = self.tcp_listener.accept() => {
                    match r {
                        Ok((socket, remote_addr)) => {
                            trace!("{INNER_NAME}: new client: {remote_addr}");
                            spawn_inbound(Box::new(socket), remote_addr.to_string(), self.sender.clone());
                        },
                        Err(err) => {
                            error!("{INNER_NAME}: error accepting: {}", err);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// To be called inside a separate task if we want to handle concurrency
    pub async fn connect(&self, ctk: CancellationToken, si: SendInfo) -> Result<(), anyhow::Error> {
        debug!("{INNER_NAME}: Connecting to: {}", si.addr);
        // "bt:<MAC>" = phone found over BLE, reached over Bluetooth RFCOMM.
        #[cfg(windows)]
        let socket: Box<dyn crate::utils::IoStream> = match crate::hdl::rfcomm::parse_bt_addr(&si.addr) {
            Some(mac) => Box::new(crate::hdl::rfcomm::connect_stream(mac).await?),
            None => Box::new(TcpStream::connect(si.addr.clone()).await?),
        };
        #[cfg(not(windows))]
        let socket: Box<dyn crate::utils::IoStream> = Box::new(TcpStream::connect(si.addr.clone()).await?);

        let crate::OutboundPayload::Files(files) = &si.ob;
        let total_bytes: u64 = files.iter().filter_map(|f| std::fs::metadata(f).ok()).map(|m| m.len()).sum();
        let mut or = OutboundRequest::new(
            self.endpoint_id,
            socket,
            si.id,
            self.sender.clone(),
            si.ob,
            RemoteDeviceInfo {
                device_type: crate::DeviceType::Unknown,
                name: si.name,
            },
        );

        // Over Bluetooth, move payloads worth it to the phone's Wi-Fi: joining
        // its network takes a few seconds, more than 1 MB takes longer than that.
        #[cfg(windows)]
        if crate::hdl::rfcomm::parse_bt_addr(&si.addr).is_some() && total_bytes > 1024 * 1024 {
            or.enable_wifi_upgrade();
        }

        // Send connection request
        or.send_connection_request().await?;
        // Send UKEY init
        or.send_ukey2_client_init().await?;

        loop {
            tokio::select! {
                _ = ctk.cancelled() => {
                    info!("{INNER_NAME}: tracker cancelled, breaking");
                    break;
                },
                r = or.handle() => {
                    if let Err(e) = r {
                        match e.downcast_ref() {
                            Some(AppError::NotAnError) => break,
                            None => {
                                if or.state.state == State::Initial {
                                    break;
                                }

                                if or.state.state != State::Finished && or.state.state != State::Cancelled {
                                    let _ = self.sender.clone().send(ChannelMessage {
                                        id: si.addr,
                                        direction: ChannelDirection::LibToFront,
                                        state: Some(State::Disconnected),
                                        ..Default::default()
                                    });
                                }
                                error!("{INNER_NAME}: error while handling client: {e} ({:?})", or.state.state);
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Runs the receive side of one connection (TCP or Bluetooth) until it ends.
pub fn spawn_inbound(socket: Box<dyn crate::utils::IoStream>, remote_addr: String, sender: Sender<ChannelMessage>) {
    tokio::spawn(async move {
        let mut ir = InboundRequest::new(socket, remote_addr.clone(), sender.clone());

        loop {
            match ir.handle().await {
                Ok(_) => {}
                Err(e) => match e.downcast_ref() {
                    Some(AppError::NotAnError) => break,
                    None => {
                        if ir.state.state == State::Initial {
                            break;
                        }

                        if ir.state.state != State::Finished {
                            let _ = sender.send(ChannelMessage {
                                id: remote_addr.clone(),
                                direction: ChannelDirection::LibToFront,
                                state: Some(State::Disconnected),
                                ..Default::default()
                            });
                        }
                        error!("{INNER_NAME}: error while handling client: {e} ({:?})", ir.state.state);
                        break;
                    }
                },
            }
        }
    });
}
