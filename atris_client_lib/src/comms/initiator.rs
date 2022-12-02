use std::sync::Arc;

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use webrtc::{
    data_channel::RTCDataChannel, peer_connection::sdp::session_description::RTCSessionDescription,
};

use super::signal;

use super::{AtrisChannel, AtrisConnection};

pub struct AtrisInitiator {
    connection: AtrisConnection,
    local_description: RTCSessionDescription,
    data_channel: Arc<RTCDataChannel>,
}
impl AtrisInitiator {
    /// Create a new initiator
    pub async fn new(mut connection: AtrisConnection) -> Result<Self> {
        let peer_connection = &mut connection.connection;

        // Create a datachannel with label 'data'
        let data_channel = peer_connection.create_data_channel("data", None).await?;

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
        if let Some(local_description) = peer_connection.local_description().await {
            Ok(Self {
                connection,
                local_description,
                data_channel,
            })
        } else {
            println!("");
            panic!();
            // Err(anyhow::Error::new())
        }
    }

    pub async fn close(self)->Result<(),webrtc::Error>{
        self.connection.connection.close().await
    }

    pub fn encoded_local_description(&self) -> Result<String> {
        let json_str = serde_json::to_string(&self.local_description)?;
        let b64 = signal::encode(&json_str);
        Ok(b64)
    }
    /// If we created an initiator, feed the responder's response here
    pub async fn into_channel_with<T>(self, responder_string: &String) -> Result<AtrisChannel<T>>
    where
        T: Serialize + Send + Sync + 'static,
        for<'d> T: Deserialize<'d>,
    {
        let decoded_responder_string = signal::decode(responder_string.as_str())?;
        // Convert the json input into a useful datatype
        let responder_description =
            serde_json::from_str::<RTCSessionDescription>(&decoded_responder_string)?;
        // dbg!(&responder_description);

        // Apply the answer as the remote description
        self.connection
            .connection
            .set_remote_description(responder_description)
            .await?;

        // Convert that channel into an AtrisChannel
        let channel = AtrisChannel::new(self.connection, self.data_channel);
        Ok(channel)
    }
}
