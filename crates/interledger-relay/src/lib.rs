#![forbid(unsafe_code)]

pub mod app;
mod client;
mod middlewares;
mod routes;
mod services;
#[cfg(test)]
mod testing;

use std::borrow::Borrow;

use futures::prelude::*;

pub use self::client::{Client, ClientBuilder};
pub use self::middlewares::{AuthToken};
//pub use self::services::relay::{NextHop, Route};
pub use self::routes::{NextHop, Route};

// TODO relay ilp-peer-name, ilp-destination?
// TODO Limit max ilp packet (or http body) length
// TODO limit max requests (parallel and total) per http2 connection

pub trait Service<Req: Request>: Clone {
    type Future: 'static + Send + Future<
        Item = ilp::Fulfill,
        Error = ilp::Reject,
    >;

    fn call(self, request: Req) -> Self::Future;
}

pub trait Request: Into<ilp::Prepare> + Borrow<ilp::Prepare> {}
impl Request for ilp::Prepare {}
