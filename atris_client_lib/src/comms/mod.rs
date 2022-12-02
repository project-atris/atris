use std::convert::Infallible;

use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::error::{SendError, TryRecvError};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::RTCPeerConnection;

use anyhow::{Ok, Result};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;

pub(self) type Message = String;

pub mod initiator;
pub mod responder;
pub mod signal;

/// Datatype that handles communication between two clients
pub struct AtrisConnection {
    connection: Arc<RTCPeerConnection>,
    done_reciever: Receiver<()>,
}

impl AtrisConnection {
    pub async fn new() -> Result<Self> {
        // Create a MediaEngine object to configure the supported codec
        let mut m = MediaEngine::default();

        // Register default codecs
        m.register_default_codecs()?;

        // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
        // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
        // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
        // for each PeerConnection.
        let mut registry = Registry::new();

        // Use the default set of Interceptors
        registry = register_default_interceptors(registry, &mut m)?;

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        // Prepare the configuration
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()], //consider switching to stun3
                ..Default::default()
            }],
            ..Default::default()
        };

        // Create a new RTCPeerConnection
        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        // Generate read and write
        let (done_tx, done_reciever) = tokio::sync::mpsc::channel::<()>(1);
        // Set the handler for Peer connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection.on_peer_connection_state_change(Box::new(
            move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {}", s);

                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    println!("Peer Connection has gone to failed exiting");
                    let _ = done_tx.try_send(());
                }

                Box::pin(async {})
            },
        ));
        Ok(Self {
            connection: peer_connection,
            done_reciever,
        })
    }
}

pub struct AtrisChannel<T> {
    connection: AtrisConnection,
    data_channel: Arc<RTCDataChannel>,
    sender: Sender<T>,
    receiver: Receiver<T>,
}
impl<T> AtrisChannel<T>
where
    T: Serialize + Send + Sync + 'static,
    for<'a> T: Deserialize<'a>,
{
    pub fn new(connection: AtrisConnection, data_channel: Arc<RTCDataChannel>) -> Self {
        // The channel that messages *to* this initiator will use
        let (incoming_sender, incoming_receiver) = tokio::sync::mpsc::channel::<T>(20);
        // The channel that messages *from* this initiator will use
        let (outgoing_sender, mut outgoing_receiver) = tokio::sync::mpsc::channel::<T>(20);

        // Register channel opening handling
        let arc_data_channel = Arc::clone(&data_channel);
        data_channel.on_open(Box::new(move || {
            //THIS IS WHERE THE THINGS ARE GENERATED
            println!(
                "Data channel '{}'-'{}' open.",
                arc_data_channel.label(),
                arc_data_channel.id()
            );
            Box::pin(async move {
                let mut result = Result::<usize>::Ok(0);
                // Get the next outgoing message from the `outgoing_sender`
                while let Some(next_message) = outgoing_receiver.recv().await {
                    // Send the next outgoing message and record the result
                    if let Result::Ok(msg) = bincode::serialize::<T>(&next_message) {
                        // incoming_sender.blocking_send(&msg);
                        result = arc_data_channel.send(&msg.into()).await.map_err(Into::into);
                    }
                    // unimplemented!("Sending messages");
                    // result = arc_data_channel.send(next_message).await.map_err(Into::into);
                }
            })
        }));

        // Register text message handling
        let _d_label = data_channel.label().to_owned();
        let incoming_sender = Arc::new(incoming_sender);
        data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
            // let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
            // println!("Message from DataChannel '{}': '{}'", d_label, msg_str);
            let incoming_sender = Arc::clone(&incoming_sender);
            Box::pin(async move {
                if let Result::Ok(msg) = bincode::deserialize::<T>(&msg.data) {
                    incoming_sender.send(msg).await;
                };
            })
        }));
        Self {
            connection,
            data_channel,
            sender: outgoing_sender,
            receiver: incoming_receiver,
        }
    }

    pub async fn send(&mut self, t: T) -> Result<(), SendError<T>> {
        self.sender.send(t).await
    }

    pub fn try_receive(&mut self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
    pub async fn receive(&mut self) -> Option<T> {
        self.receiver.recv().await
    }
}
impl AtrisChannel<String> {
    pub async fn io_loop(mut self) -> Result<Infallible> {
        let mut buffer = [0; 1024];
        let mut input = tokio::io::stdin();

        loop {
            tokio::select! {
                Some(incomming_message) = self.receive() => {
                    println!("From other user: '{incomming_message}'")
                },
                Result::Ok(len) = input.read(&mut buffer) => {
                    if len > 1 {
                        if let Result::Ok(msg) = String::from_utf8(Vec::from(buffer)) {
                            self.send(msg.trim().to_owned()).await?;
                        }
                        buffer = [0;1024];
                    }
                },
                else => {

                }

            };
        }
    }
}
