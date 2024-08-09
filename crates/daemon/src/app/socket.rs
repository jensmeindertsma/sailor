use crate::configuration::Configuration;
use sail_core::socket::{SocketRequest, SocketResponse};
use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;

#[derive(Clone)]
pub struct SocketHandler {
    configuration: Arc<Configuration>,
}

impl SocketHandler {
    pub fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }
}

impl Service<SocketRequest> for SocketHandler {
    type Response = SocketResponse;
    type Error = Infallible;
    type Future = SocketHandlerFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // The socket handler has no limitations and is always able to accept
        // new connections.
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: SocketRequest) -> Self::Future {
        let response_future = Box::pin(async {
            // Do potentially some async stuff with the request
            Ok(match request {
                SocketRequest::Greeting => SocketResponse::Okay,
            })
        });

        Self::Future { response_future }
    }
}

struct SocketHandlerFuture {
    response_future:
        Pin<Box<dyn Future<Output = Result<SocketResponse, Infallible>> + Send + 'static>>,
}

impl Future for SocketHandlerFuture {
    type Output = Result<SocketResponse, Infallible>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        self.response_future.as_mut().poll(context)
    }
}
