use std::borrow::Borrow;

use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
use atris_client_lib::atris_common::cipher::{ChaCha20Poly1305, KeyInit};
use atris_client_lib::comms;
use atris_client_lib::comms::responder::AtrisResponder;
use atris_client_lib::comms::{initiator::AtrisInitiator, AtrisConnection};
use atris_client_lib::{http_auth::AtrisAuth, AtrisAuthClient};


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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use atris_client_lib::comms::{initiator::AtrisInitiator,responder::AtrisResponder, AtrisConnection};    
    // let initiator = AtrisInitiator::new(AtrisConnection::new().await?).await?;
    let (initiator,client,session)=for_user("init","init").await?;
    let unused = AtrisResponder::new().await?;
    //let stdin = io::stdin(); // We get `Stdin` here.
    //stdin.read_line(&mut buffer)?;

    // println!("Initiator Description: ");
    // atris_client_lib::comms::signal::print_in_chunks(&initiator.encoded_local_description()?);
    let responder_str = "";//atris_client_lib::comms::signal::must_read_stdin()?;
    let room_code = comms::signal::read_in_line()?;// atris_client_lib::comms::signal::must_read_stdin()?;
    let room_id: u16 = room_code.parse()?;
    let mut cipher = ChaCha20Poly1305::new(session.session_id.borrow());
    let join_room_response = client.join_room(session.session_id, room_id).await??;
    let room_data = join_room_response.room_data.decrypt(&mut cipher).unwrap();

    if responder_str == room_data.responder_string {
        println!("Same resp!")
    }else{
        // dbg!(&responder_str,&room_data.responder_string);
        println!("Diff resp!")
    }
    
    let channel = initiator.into_channel_with::<String>(&room_data.responder_string).await?;
    println!("Starting loop: ");

    channel.io_loop().await?;

    Ok(())
    //original().await
}