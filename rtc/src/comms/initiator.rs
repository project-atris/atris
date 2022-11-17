use anyhow::{Result, Ok};
use serde::{Serialize, Deserialize};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use super::{AtrisConnection,AtrisChannel};

pub struct AtrisInitiator {
    connection:AtrisConnection,
    local_description: RTCSessionDescription
}
impl AtrisInitiator{
    /// Create a new initiator
    pub async fn new(mut connection:AtrisConnection) -> Result<Self> {
        let peer_connection = &mut connection.connection;
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
            Ok(Self{ connection ,local_description })
        } else {
            println!("");
            panic!();
            // Err(anyhow::Error::new())
        }

    }
        
    /// If we created an initiator, feed the responder's response here
    pub async fn into_channel_with<T>(self, responder_string: String) -> Result<AtrisChannel<T>>
        where T:Serialize+Send+Sync+'static,
        for<'d> T:Deserialize<'d>
    {
        // Convert the json input into a useful datatype
        let responder_description = serde_json::from_str::<RTCSessionDescription>(&responder_string)?;

        // Apply the answer as the remote description
        self.connection.connection.set_remote_description(responder_description).await?;

        // Create a datachannel with label 'data'
        let data_channel = self.connection.connection.create_data_channel("data", None).await?;

        // Convert that channel into an AtrisChannel
        let channel = AtrisChannel::new(self.connection, data_channel);

        Ok(channel)
    }    
}