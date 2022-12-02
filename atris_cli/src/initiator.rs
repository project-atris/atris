use std::io::{stdin, stdout};

use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
use atris_client_lib::comms::responder::AtrisResponder;
use atris_client_lib::comms::{initiator::AtrisInitiator, AtrisConnection};
use atris_client_lib::{http_auth::AtrisAuth, AtrisAuthClient};
use std::io::Write;

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
        let fake_initiator_string = initiator.encoded_local_description()?;
        let auth = client
            .authenticate_user(user, pass, &fake_initiator_string)
            .await??;
        Ok((initiator, client, auth))
    }
    // Create the client to the authorization server
    let (init, client, session) = for_user("marcel", "test").await?;
    init.close().await?;
    
    let mut out = stdout();
    write!(out,"Who do you want to talk to? ");
    out.flush();

    let other_username = stdin().lines().next().ok_or("No terminal input!")??;
    let room = client
    .create_room(session.session_id.clone(), &other_username)
    .await??;
    println!("Ask them to join you!\nRoom ID: {}", room.room_id);
    let responder = AtrisResponder::new().await?;
    let (responder_string, channel_future) = responder
        .open_channel_with::<String>(&room.initiator_string)
        .await?;
    // dbg!(&room.initiator_string);
    let room_symm_key =
        client
            .set_room_responder(
                room.room_id,
                session.session_id,
                &other_username,
                &responder_string
            )
            .await
    ??;

    let mut out = stdout();
    writeln!(out,"Room symmetric key received");
    out.flush();

    let channel = channel_future.await.ok_or("No channel recieved!")?;

    let mut out = stdout();
    writeln!(out,"Connection established!");
    out.flush();

    channel.io_loop().await?;
    Ok(())
}