use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::RTCPeerConnection;

use anyhow::{Result, Ok};
use tokio::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::math_rand_alpha;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::symmetric_provided::EncryptedBytes;

/// Datatype that handles communication between two clients
pub struct AtrisConnection {
    connection: Arc<RTCPeerConnection>,
    data_channel: Arc<RTCDataChannel>,
    //received_messages:Arc<Vec<String>>,
    message_sender:Sender<String>,
    received_messages:Receiver<String>,
    done_rx: Receiver<()>,
}
impl AtrisConnection {
    pub fn send(&mut self,s:String) {
        // self.data_channel.send_text(s);
        self.message_sender.blocking_send(s);
    }
}
impl AtrisConnection {
    /// Create a new initiator
    pub async fn new_initiator() -> Result<AtrisConnection> {
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
        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

        // Set the handler for Peer connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {}", s);

                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    println!("Peer Connection has gone to failed exiting");
                    let _ = done_tx.try_send(());
                }

                Box::pin(async {})
            }))
            .await;

        // Create a datachannel with label 'data'
        let data_channel = peer_connection.create_data_channel("data", None).await?;

        let (incoming_sender,incoming_receiver) = tokio::sync::mpsc::channel::<String>(20);
        let (outgoing_sender,outgoing_receiver) = tokio::sync::mpsc::channel::<String>(20);

        // Register channel opening handling
        let arc_data_channel = Arc::clone(&data_channel);
        data_channel.on_open(Box::new(move || { //THIS IS WHERE THE THINGS ARE GENERATED
            println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds", arc_data_channel.label(), arc_data_channel.id());
            Box::pin(async move {
                let mut result = Result::<usize>::Ok(0);
                while let Some(next_message) = outgoing_receiver.blocking_recv() {
                    result = arc_data_channel.send_text(next_message).await.map_err(Into::into);
                }
            })
        })).await;

        // Register text message handling
        let d_label = data_channel.label().to_owned();
        data_channel
            .on_message(Box::new(move |msg: DataChannelMessage| {
                let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                println!("Message from DataChannel '{}': '{}'", d_label, msg_str);
                incoming_sender.send(msg_str);
                Box::pin(async {})
            }))
            .await;

        // Create an offer to send to the browser
        let offer = peer_connection.create_offer(None).await?;

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        peer_connection.set_local_description(offer).await?;

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        // -- In understandable words: I think this waits until we get our IP info that we forward to other users --
        let _ = gather_complete.recv().await;

        // Output the answer in base64 so we can paste it in browser
        // -- Actually, don't convert to base64 because raw json is smaller in size --
        if let Some(local_desc) = peer_connection.local_description().await {
            let json_str = serde_json::to_string(&local_desc)?;
            //let b64 = signal::encode(&json_str);
            println!("{}", json_str);
            //println!("{}", b64);
        } else {
            println!("generate local_description failed!");
        }

        Ok(AtrisConnection { message_sender:outgoing_sender,connection: peer_connection,data_channel, received_messages:incoming_receiver, done_rx, message_sender: todo!() })
    }

    
    
    /// If we created a initiator, feed the responder's response here
    pub async fn set_responder(&mut self, line: String) -> Result<()> {

        // Wait for the answer to be pasted
        //let line = signal::must_read_stdin()?;
        // let desc_data = signal::decode(line.as_str())?;
        //let answer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;

        // Convert the json input into a useful datatype
        let answer = serde_json::from_str::<RTCSessionDescription>(&line)?;

        // Apply the answer as the remote description
        self.connection.set_remote_description(answer).await?;


        // I think this starts the actual data transfer, so lets do some testing here
        println!("Press ctrl-c to stop");
        tokio::select! {
            _ = self.done_rx.recv() => {
                println!("received done signal!");
            }
            _ = tokio::signal::ctrl_c() => {
                println!("");
            }
        };

        self.connection.close().await?;

        Ok(())

    }



    /// Create a new client
    pub async fn new_client() -> Result<AtrisConnection> {
        
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
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        // Create a new RTCPeerConnection
        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

        // Set the handler for Peer connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {}", s);

                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    println!("Peer Connection has gone to failed exiting");
                    let _ = done_tx.try_send(());
                }

                Box::pin(async {})
            }))
            .await;

        let (data_channel_sender, mut data_channel_receiver) = tokio::sync::mpsc::channel::<Arc<RTCDataChannel>>(1);
        // Register data channel creation handling
        peer_connection
            .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
                let d_label = d.label().to_owned();
                let d_id = d.id();
                println!("New DataChannel {} {}", d_label, d_id);

                // Register channel opening handling
                Box::pin(async move {
                    let d2 = Arc::clone(&d);
                    let d_label2 = d_label.clone();
                    let d_id2 = d_id;
                    d.on_open(Box::new(move || {
                        println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds", d_label2, d_id2);

                        Box::pin(async move {
                            let mut result = Result::<usize>::Ok(0);
                            while result.is_ok() {
                                let timeout = tokio::time::sleep(Duration::from_secs(5));
                                tokio::pin!(timeout);

                                tokio::select! {
                                    _ = timeout.as_mut() =>{
                                        let message = math_rand_alpha(15);
                                        println!("Sending '{}'", message);
                                        result = d2.send_text(message).await.map_err(Into::into);
                                    }
                                };
                            }
                        })
                    })).await;

                    // Register text message handling
                    d.on_message(Box::new(move |msg: DataChannelMessage| {
                        let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                        println!("Message from DataChannel '{}': '{}'", d_label, msg_str);
                        Box::pin(async {})
                    })).await;
                })
            }))
            .await;


        Ok(AtrisConnection { connection: peer_connection, done_rx: done_rx, data_channel: todo!(), message_sender: todo!(), received_messages: todo!() })
    }


    /// If we created a client, set the server
    pub async fn set_server(&mut self, line: String) -> Result<()> {

        // Wait for the offer to be pasted
        let offer = serde_json::from_str::<RTCSessionDescription>(&line)?;

        // Set the remote SessionDescription
        self.connection.set_remote_description(offer).await?;

        // Create an answer
        let answer = self.connection.create_answer(None).await?;

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = self.connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        self.connection.set_local_description(answer).await?;

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        // Output the answer in base64 so we can paste it in browser
        // -- no, just print the thing normally --
        if let Some(local_desc) = self.connection.local_description().await {
            let json_str = serde_json::to_string(&local_desc)?;
            println!("{}", json_str);
        } else {
            println!("generate local_description failed!");
        }

        println!("Press ctrl-c to stop");
        tokio::select! {
            _ = self.done_rx.recv() => {
                println!("received done signal!");
            }
            _ = tokio::signal::ctrl_c() => {
                println!("");
            }
        };

        self.connection.close().await?;

        Ok(())
    }


}
