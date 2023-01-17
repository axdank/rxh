mod files;
mod proxy;

use std::{future::Future, net::SocketAddr, pin::Pin};

use hyper::{body::Incoming, service::Service, Request};

use self::files::send_file;
use crate::{
    config,
    http::{
        request::ProxyRequest,
        response::{BoxBodyResponse, LocalResponse},
    },
};

/// Implements [`Service`] and handles incoming requests.
pub(crate) struct Rxh {
    /// Reference to the configuration of this [`crate::server::Server`]
    /// instance.
    config: &'static config::Server,

    // Socket address of the connected client.
    client_addr: SocketAddr,

    // Listening socket address.
    server_addr: SocketAddr,
}

impl Rxh {
    /// Creates a new [`Rxh`] service.
    pub fn new(
        config: &'static config::Server,
        client_addr: SocketAddr,
        server_addr: SocketAddr,
    ) -> Self {
        Self {
            config,
            client_addr,
            server_addr,
        }
    }
}

impl Service<Request<Incoming>> for Rxh {
    type Response = BoxBodyResponse;

    type Error = hyper::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, request: Request<Incoming>) -> Self::Future {
        let Rxh {
            client_addr,
            server_addr,
            config,
        } = *self;

        Box::pin(async move {
            if !request.uri().to_string().starts_with(&config.prefix) {
                return Ok(LocalResponse::not_found());
            }
            match &config.kind {
                config::Kind::Proxy(ref proxy) => {
                    let request = ProxyRequest::new(request, client_addr, server_addr);
                    proxy::forward(request, proxy.target).await
                }
                config::Kind::Static(ref config) => {
                    Ok(send_file(&request.uri().path()[1..], config).await)
                }
            }
        })
    }
}
