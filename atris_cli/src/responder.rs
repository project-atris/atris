use std::borrow::Borrow;
use std::io::stdin;

use atris_client_lib::atris_common::cipher::KeyInit;
use atris_client_lib::atris_common::{
    authenticate_user::AuthenticateUserResponse, cipher::ChaCha20Poly1305,
};

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
    let (intitator, client, session) = for_user("terrior", "password").await?;

    print!("Please provide the room key: ");
    let room_key = stdin().lines().next().ok_or("No terminal input!")??;
    let room_id: u16 = room_key.parse()?;
    //let room = dbg!(terrior2
    //    .create_room(terrior2_session.session_id.clone(), "terrior").await)??;

    let mut cipher = ChaCha20Poly1305::new(session.session_id.borrow());
    let join_room_response = client.join_room(session.session_id, room_id).await??;

    let room_data = join_room_response.room_data.decrypt(&mut cipher).unwrap();

    let channel = intitator
        .into_channel_with::<String>(&room_data.responder_string)
        .await?;

    channel.io_loop();
    Ok(())
}
