use std::task::{Context, Poll};

use axum::{extract::Request, response::Response};
use tower::Service;

use crate::provider::ProviderService;

#[derive(Clone, Default)]
pub struct GatewayService {
    provider: ProviderService,
}

impl Service<Request> for GatewayService {
    type Response = Response;
    type Error = <ProviderService as Service<Request>>::Error;
    type Future = <ProviderService as Service<Request>>::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.provider.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        self.provider.call(request)
    }
}
