use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
use atris_client_lib::comms::AtrisChannel;
use atris_client_lib::comms::responder::AtrisResponder;
use atris_client_lib::comms::{initiator::AtrisInitiator, AtrisConnection};
use atris_client_lib::{http_auth::AtrisAuth, AtrisAuthClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    async fn for_user(
        user: &str,
        pass: &str,
    ) -> Result<
        (AtrisInitiator, AtrisAuth, AuthenticateUserResponse),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let client = AtrisAuth::new()?;
        client.create_user(user, pass).await?;
        let initiator = AtrisInitiator::new(AtrisConnection::new().await?).await?;
        let initiator_string = initiator.encoded_local_description()?;
        let auth = client
            .authenticate_user(user, pass, &initiator_string)
            .await??;
        Ok((initiator, client, auth))
    }
    // Create the client to the authorization server
    let (terrior_initiator, _terrior, _terrior_session) = for_user("terrior", "password").await?;
    let (_terrior2_initiator, terrior2, terrior2_session) =
        for_user("terrior2", "password").await?;
    let terrior2_responder = AtrisResponder::new().await?;
    let room = dbg!(
        terrior2
            .create_room(terrior2_session.session_id.clone(), "terrior")
            .await
    )??;
    let (terrior2_responder_string, terrior2_channel) = terrior2_responder
        .into_channel_parts_with::<String>(&room.initiator_string)
        .await?;
    let set_room = dbg!(
        terrior2
            .set_room_responder(
                room.room_id,
                terrior2_session.session_id,
                "terrior",
                &terrior2_responder_string
            )
            .await
    )??;
    let mut terrior_parts = terrior_initiator
        .into_channel_parts_with::<String>(&terrior2_responder_string)
        .await?;
    let mut terrior_channel = AtrisChannel::new(terrior_parts, set_room.room_symmetric_key.as_cipher());
    let mut terrior2_channel =  AtrisChannel::new(terrior2_channel.await.ok_or("Ew!")?, set_room.room_symmetric_key.as_cipher());

    terrior_channel.send("From terrior".into()).await;
    dbg!(terrior2_channel.receive().await);

    terrior2_channel.send("From terrior2".into()).await;
    dbg!(terrior_channel.receive().await);

    dbg!(room.room_id, set_room.room_symmetric_key);
    Ok(())
}
