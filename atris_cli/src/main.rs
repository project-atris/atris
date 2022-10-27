use atris_client_lib::{AtrisAuthClient,http_auth::AtrisAuth};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create the client to the authorization server
    let client = AtrisAuth::new()?;
    // Send the request to the server
    //let user = client.create_user("usernames", "password-secret-shh").await;
    //dbg!(user);
    Ok(())
}
