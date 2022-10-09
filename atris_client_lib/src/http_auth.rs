use std::fmt::Debug;

use async_trait::async_trait;
use atris_common::{
    authenticate_user::{AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserResponse},
    create_user::{CreateUserError, CreateUserRequest, CreateUserResponse}, REGION,
};

use reqwest::{RequestBuilder, Method, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use aws_config::meta::region::RegionProviderChain;

use crate::{AtrisAuthClient, InvocationResult, InvocationError};

/// The API of the Atris authentication server, implemented using http requests.
/// This bundles all of the functions necessary for user creation and authentication, as well as initiating the key exchange.
/// 
/// This version should be better as it will use the authentication headers instead of plaintext
/// Once complete sdk_auth will be deprecated
/// ```
/// use atris_client_lib::{AtrisAuthClient,http_auth::AtrisAuth};
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     /// Create the client to the authorization server
///     let client = AtrisAuth::new()?;
///     // Send the request to the server
///     let user = client.create_user("username", "password-secret-shh").await;
///     Ok(())
/// }
/// ```
pub struct AtrisAuth {
    /// The http client that this client will use for API calls
    client: reqwest::Client
}

impl AtrisAuth {
    /// Create an [`AtrisAuth`] from the environment variable configurations
    pub fn new() -> Result<Self,reqwest::Error> {
        let client = reqwest::Client::builder().user_agent("atris_client_lib").build()?;
        Ok(Self {
            client
        })
    }
}
#[async_trait]
impl AtrisAuthClient for AtrisAuth{
    type Error = reqwest::Error;
    type FunctionIdentifier = &'static str;

    type BaseResponse=Response;

    const CREATE_USER_FN:Self::FunctionIdentifier = "https://6mfd7yxy3tibberkbtkmjbhnsu0wevek.lambda-url.us-west-2.on.aws";
    // const CREATE_USER_FN:Self::FunctionIdentifier = "http://127.0.0.1:9000";
    const AUTHENTICATE_USER_FN:Self::FunctionIdentifier = "https://y46vbul2oe7qumkca6rkyi7k7a0aixvu.lambda-url.us-west-2.on.aws/";

    async fn invoke_lambda<'s, P: Serialize+Sync, R:DeserializeOwned>(
        &'s self,
        lambda_function_url: Self::FunctionIdentifier,
        payload: &'s P,
    ) -> InvocationResult<R,Self::Error>{
        // Get the payload as a `String`
        Ok(self.client.post(lambda_function_url).json(payload).send()
            .await?.json().await?)
        // dbg!("Sending");
        // let text = self.client.post(lambda_function_url).json(payload).send()

        //     .await?.text().await?;
        // dbg!("Sent");
        // dbg!(text);
        // todo!()
    }


}