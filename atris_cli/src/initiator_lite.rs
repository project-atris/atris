use std::io::{stdin, stdout};

use atris_client_lib::atris_common::Cipher;
use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
use atris_client_lib::atris_common::cipher::KeyInit;
use atris_client_lib::comms::AtrisChannel;
use atris_client_lib::comms::responder::AtrisResponder;
use atris_client_lib::comms::{initiator::AtrisInitiator, AtrisConnection};
use atris_client_lib::{http_auth::AtrisAuth, AtrisAuthClient};
use std::io::Write;

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
    println!("Authenticated");
    Ok((initiator, client, auth))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (unused,client,session)=for_user("resp","resp").await?;
    let comm = AtrisResponder::new().await?;
    let room = client
        .create_room(session.session_id.clone(), "init")
    .await??;
    let (b64,channel) = comm.into_channel_parts_with::<String>(&room.initiator_string).await?;
    let room_key = client
        .set_room_responder(room.room_id,session.session_id,"init",&b64)
    .await??;
    println!("Ask them to join you!\nRoom ID: {}", room.room_id);

    AtrisChannel::new(channel.await.ok_or("No channel recieved!")?,room_key.room_symmetric_key.as_cipher()).io_loop().await?;

    Ok(())
}
