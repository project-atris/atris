pub mod auth_table;
pub mod room_table;
pub mod session_table;

// pub struct AtrisRequest<R>{
//     pub payload: R,
//     pub headers: lambda_http::http::header::HeaderMap,
// }
// // pub type AtrisEvent<R> = Result<AtrisRequest<R>,serde_json::Error>;

// pub struct AtrisLambda<S,R>{
//     service: S,
//     phantom: PhantomData<R>
// }
// impl <R,S:Service<AtrisRequest<R>>> AtrisLambda<S,R>{
//     pub fn new(service:S)->Self{
//         Self {
//             service,
//             phantom:PhantomData
//         }
//     }
// }

// pub struct AtrisFuture<F>(Result<Pin<Box<F>>,serde_json::Error>);

// impl <F:Future>Future for AtrisFuture<F>
//     where F::Output:Serialize
// {
//     // type Output = Result<F::Output,AtrisError>;
//     type Output = Result<String,AtrisError>;
//     fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
//         match &mut self.0 {
//             Ok(future)=>{
//                 future.as_mut().poll(cx).map(|r|{
//                     Ok(serde_json::to_string(&r).unwrap_or_default())
//                 })
//                 // Pin::new(future.as_mut()).poll(cx).map(Ok)
//             },
//             Err(err)=>Poll::Ready(Err(AtrisError::SerdeError(format!("{}",err))))
//         }
//     }
// }

// impl <R,S:Service<AtrisRequest<R>>> Service<lambda_http::Request> for AtrisLambda<S,R>
//     where for <'d> R:Deserialize<'d>,
//     S::Response:Serialize,
//     S::Error:Serialize
// {
//     // type Response = Result<S::Response,S::Error>;
//     type Response = String;

//     type Error = AtrisError;

//     type Future= AtrisFuture<S::Future>;

//     fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, req: lambda_http::Request) -> Self::Future {
//         let new_req =serde_json::from_slice::<R>(req.body()).map(|request|{
//             let request = AtrisRequest{
//                 payload:request,
//                 headers:req.headers().clone()
//             };
//             Box::pin(self.service.call(request))
//         });
//         AtrisFuture(new_req)
//     }
// }

/// Generates the main function for a lambda which uses the provided function as its handler
#[macro_export]
macro_rules! run_lambda_http {
    (|$request_name:ident : Request<$request:ty>| -> $ret:ty $block:block) => {
        use lambda_http::Error as LambdaError;
        #[tokio::main]
        async fn main() -> Result<(), LambdaError> {
            // use std::convert::Infallible;
            async fn handler($request_name: lambda_http::http::Request<$request>) -> $ret {
                $block
            };
            async fn handler_wrapper(
                request: lambda_http::Request,
            ) -> Result<lambda_http::Response<lambda_http::Body>, serde_json::Error> {
                let (parts, body) = request.into_parts();
                let parsed_request = serde_json::from_slice::<$request>(&body)?;
                let new_request = lambda_http::http::Request::from_parts(parts, parsed_request);
                let result = handler(new_request).await;
                let body_text = serde_json::to_string(&result)?;
                Ok(lambda_http::Response::new(lambda_http::Body::Text(
                    body_text,
                )))
            };
            lambda_http::run(lambda_http::service_fn(handler_wrapper)).await?;
            Ok(())
        }
    };
}

/// Generates the main function for a lambda which uses the provided function as its handler
#[macro_export]
macro_rules! run_lambda {
    (|$request_name:ident : $request:ty $(,$header:ident)?| -> $ret:ty $block:block) => {
        // use lambda_runtime::Error as LambdaError;
        // use std::convert::Infallible;
        // use atris_server::AtrisLambda;
        // #[tokio::main]
        // async fn main() -> Result<(), LambdaError> {
        //     use lambda_runtime::{self, service_fn};
        //     use atris_server::Statused;
        //     async fn internal_handler($request_name: $request) -> $ret {
        //         $block
        //     };
        //     async fn technnicaly_infallible_handler(event: $request) -> Result<Statused, Infallible> {
        //     // async fn technnicaly_infallible_handler(event: $request) -> Result<Statused<$ret>, Infallible> {
        //         Ok(Statused::new(internal_handler(event).await))
        //     }
        //     lambda_runtime::run(service_fn(technnicaly_infallible_handler)).await?;
        //     Ok(())
        // }

        use lambda_runtime::{service_fn, Error,Service};
        use serde_json::{json, Value};
        use std::convert::Infallible;
        use log::LevelFilter;
        use lambda_http::Request;
        #[tokio::main]
        async fn main() -> Result<(), Error> {
            tracing_subscriber::fmt()
                // Configure formatting settings.
                .with_target(false)
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .with_level(true)
                // Set the subscriber as the default.
                .init();
            let service_handle = service_fn(handler);
            lambda_runtime::run(service_handle).await?;
            Ok(())
        }

        async fn func_internal($request_name: $request) -> $ret {
            $block
        }
        async fn handler($request_name: LambdaEvent<Request>) -> Result<String,Infallible> {
            let res = func_internal($request_name).await;
        }
    };
}
