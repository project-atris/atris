use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
use atris_client_lib::comms::responder::AtrisResponder;
use atris_client_lib::{http_auth::AtrisAuth, comms::initiator,AtrisAuthClient};
use atris_client_lib::comms::{signal,initiator::AtrisInitiator, AtrisConnection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("here");
    let comm = AtrisInitiator::new(AtrisConnection::new().await?).await?;
    println!("here1");
    //let stdin = io::stdin(); // We get `Stdin` here.
    //stdin.read_line(&mut buffer)?;
    println!("Initiator Description: ");
    crate::signal::print_in_chunks(&comm.encoded_local_description()?);
    let responder_str = crate::signal::must_read_stdin()?;
    println!("here2");
    let channel = comm.into_channel_with::<String>(&responder_str).await?;
    println!("here3");

    channel.io_loop().await?;
    println!("here4");

    Ok(())
    //original().await
}
