use std::convert::Infallible;

use std::fmt::{Display, Debug};
use std::marker::PhantomData;
use std::sync::Arc;
use atris_common::cipher::KeyInit;
use atris_common::{CipherKey, Encrypted, Cipher, EncryptionError};
use tokio::io::AsyncReadExt;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TryRecvError;
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

pub struct AtrisChannelParts<T> {
    connection: AtrisConnection,
    data_channel: Arc<RTCDataChannel>,
    sender: Sender<Encrypted<T>>,
    receiver: Receiver<Encrypted<T>>,
}

pub struct AtrisChannel<T> {
    atris_channel_internal: AtrisChannelParts<T>,
    cipher: Cipher,
    phantom_data:PhantomData<T>
}
impl <T> Debug for AtrisChannel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtrisChannel").finish()
    }
}
impl<T> AtrisChannelParts<T>
where
    T: Serialize + Send + Sync + 'static,
    for<'a> T: Deserialize<'a>,
{
    pub fn new(connection: AtrisConnection, data_channel: Arc<RTCDataChannel>) -> Self {
        // The channel that messages *to* this initiator will use
        let (incoming_sender, incoming_receiver) = tokio::sync::mpsc::channel(20);
        // The channel that messages *from* this initiator will use
        let (outgoing_sender, mut outgoing_receiver) = tokio::sync::mpsc::channel(20);

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
                    if let Result::Ok(msg) = bincode::serialize(&next_message) {
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
                if let Result::Ok(msg) = bincode::deserialize(&msg.data) {
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
}

#[derive(Debug)]
pub enum SendError<T>{
    ChannelError(T),
    EncryptionError(T,EncryptionError)
}
impl <T:Debug> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
impl <T:Debug> std::error::Error for SendError<T> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
#[derive(Debug)]
pub enum RecieveError<T>{
    None,
    DecryptionError(T,EncryptionError)
}
#[derive(Debug)]
pub enum TryReceiveError{
    Emp(TryRecvError),
    DecryptionError(EncryptionError)
}
impl <T> AtrisChannel<T>
where
    T: Serialize + Send + Sync + 'static,
    for<'a> T: Deserialize<'a>
{
    pub fn new(parts:AtrisChannelParts<T>,cipher:Cipher)->Self{
        Self {
            phantom_data:PhantomData,
            atris_channel_internal:parts,
            cipher
        }
    }

    pub async fn send(&mut self, t: T) -> Result<(), SendError<T>> {
        let encrypted = match Encrypted::encrypt(&t, &mut self.cipher){
            Result::Ok(s)=>s,
            Err(e)=>return Err(SendError::EncryptionError(t, e))
        };
        self.atris_channel_internal.sender.send(encrypted).await.map_err(|e|{
            SendError::ChannelError(t)
        })
    }

    pub fn try_receive(&mut self) -> Result<T, TryReceiveError> {
        self.atris_channel_internal.receiver.try_recv()
        .map_err(TryReceiveError::Emp)
        .and_then(|e|{
            e.decrypt(&mut self.cipher).map_err(TryReceiveError::DecryptionError)
        })
    }
    pub async fn receive(&mut self) -> Option<atris_common::Result<T>> {
        Some(self.atris_channel_internal.receiver.recv().await?.decrypt(&mut self.cipher))
    }
}



impl AtrisChannel<String> {
    pub async fn io_loop(mut self) -> Result<Infallible> {
        let mut buffer = [0; 1024];
        let mut input = tokio::io::stdin();

        loop {
            tokio::select! {
                Some(atris_common::Result::Ok(incoming_message)) = self.receive() => {
                    println!("From other user: '{incoming_message}'")
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
