use std::{future::Ready, task::Poll};

use anyhow::Error;
use axum::{extract::Request, response::Response};
use tower::Service;

#[derive(Clone, Default)]
pub struct ProviderService;

impl Service<Request> for ProviderService {
    type Response = Response;
    type Error = Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        std::future::ready(Ok(Response::new(request.into_body())))
    }
}
