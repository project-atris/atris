use std::sync::Arc;

use serde::{Serialize, Deserialize};
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

use super::{AtrisConnection,AtrisChannel};

pub struct AtrisResponder {
    connection:AtrisConnection,
}

impl AtrisResponder {
    pub async fn new()->Result<Self> {
        let connection = AtrisConnection::new().await?;        
        Ok(Self{ connection })
    }

    /// Set the initator's description
    pub async fn open_channel_with<T>(mut self, offer_str: String) -> Result<AtrisChannel<T>>
        where T:Serialize+Send+Sync+'static,
        for<'d> T:Deserialize<'d>
    {
        let peer_connection = &mut self.connection.connection;        

        // Wait for the offer to be pasted
        let offer = serde_json::from_str::<RTCSessionDescription>(&offer_str)?;

        // Set the remote SessionDescription
        peer_connection.set_remote_description(offer).await?;

        // Create an answer
        let answer = peer_connection.create_answer(None).await?;

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        peer_connection.set_local_description(answer).await?;

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        // Output the answer in base64 so we can paste it in browser
        // -- no, just print the thing normally --
        if let Some(local_desc) = peer_connection.local_description().await {
            let json_str = serde_json::to_string(&local_desc)?;
            println!("{}", json_str);
            // Ok(t)
        } else {
            println!("generate local_description failed!");
        }

        let (data_channel_sender, mut data_channel_receiver) = tokio::sync::mpsc::channel::<Arc<RTCDataChannel>>(1);
        let data_channel_sender = Arc::new(data_channel_sender);
        // Register data channel creation handling
        peer_connection
            .on_data_channel(Box::new(move |data_channel: Arc<RTCDataChannel>| {
                let data_channel_sender = Arc::clone(&data_channel_sender);
                Box::pin(async move {data_channel_sender.send(data_channel).await;})
            }));

        // println!("Press ctrl-c to stop");
        tokio::select! {
            Some(data_channel) = data_channel_receiver.recv() => {
                Ok(AtrisChannel::new(self.connection, data_channel))
            }
            else => {
                panic!("Test")
                // Err(())
            }
        }
    }

}

/*
        let (data_channel_sender, mut data_channel_receiver) = tokio::sync::mpsc::channel::<AtrisChannel>(1);
        // Register data channel creation handling
        peer_connection
            .on_data_channel(Box::new(move |data_channel: Arc<RTCDataChannel>| {
                

                
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
*/