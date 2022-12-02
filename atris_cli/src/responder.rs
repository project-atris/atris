use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
use atris_client_lib::comms::responder::AtrisResponder;
use atris_client_lib::{http_auth::AtrisAuth, comms::initiator,AtrisAuthClient};
use atris_client_lib::comms::{signal,initiator::AtrisInitiator, AtrisConnection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("here");
    let comm = AtrisResponder::new().await?;
    println!("here1");
    //let stdin = io::stdin(); // We get `Stdin` here.
    //stdin.read_line(&mut buffer)?;
    let initiator_str = crate::signal::must_read_stdin()?;
    println!("here2");
    let (res_str,channel) = comm.open_channel_with::<String>(&initiator_str).await?;
    println!("here3");
    signal::print_in_chunks(&res_str);

    channel.await.unwrap().io_loop().await?;
    println!("here4");

    Ok(())
}
