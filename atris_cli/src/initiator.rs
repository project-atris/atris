use std::io::stdin;

use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
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
    let (_init, client, session) = for_user("terrior", "password").await?;
    let responder = AtrisResponder::new().await?;
    print!("Who do you want to talk to? ");
    let other_username = stdin().lines().next().ok_or("No terminal input!")??;
    let room = dbg!(
        client
            .create_room(session.session_id.clone(), &other_username)
            .await
    )??;
    println!("Ask them to join you!\nRoom ID: {}", room.room_id);
    let (responder_string, channel_future) = responder
        .open_channel_with::<String>(&room.initiator_string)
        .await?;
    let _ = dbg!(
        client
            .set_room_responder(
                room.room_id,
                session.session_id,
                &other_username,
                &responder_string
            )
            .await
    )??;
    let channel = channel_future.await.ok_or("No channel recieved!")?;

    channel.io_loop().await?;
    Ok(())
}
