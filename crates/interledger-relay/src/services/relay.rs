use std::sync::{Arc, RwLock};

use bytes::Bytes;
use futures::future::{Either, err};
use futures::prelude::*;

use crate::{Service, Request};
use crate::client::Client;
use crate::routes::{Route, RoutingTable};

//pub type Route = routes::Route<NextHop>;

// TODO rename Relay to: RouterRelay? RelayRouter?
#[derive(Clone, Debug)]
pub struct RelayService {
    data: Arc<RelayData>,
    client: Client,
}

#[derive(Debug)]
struct RelayData {
    address: Bytes,
    routes: RwLock<RoutingTable>,
}

impl<Req> Service<Req> for RelayService
where
    Req: Request,
{
    type Future = Box<
        dyn Future<
            Item = ilp::Fulfill,
            Error = ilp::Reject,
        > + Send + 'static,
    >;

    fn call(self, request: Req) -> Self::Future {
        Box::new(self.forward(request.into()))
    }
}

impl RelayService {
    pub fn new(client: Client, routes: Vec<Route>) -> Self {
        RelayService {
            data: Arc::new(RelayData {
                address: client.address().clone(),
                routes: RwLock::new(RoutingTable::new(routes)),
            }),
            client,
        }
    }

    /// Replace the routing table.
    pub fn set_routes(&self, new_routes: Vec<Route>) {
        let mut routes = self.data.routes.write().unwrap();
        *routes = RoutingTable::new(new_routes);
    }

    fn forward(self, prepare: ilp::Prepare)
        -> impl Future<Item = ilp::Fulfill, Error = ilp::Reject>
    {
        let routes = self.data.routes.read().unwrap();
        let route = match routes.resolve(prepare.destination()) {
            Some(route) => route,
            None => return Either::B(err(self.make_reject(
                ilp::ErrorCode::F02_UNREACHABLE,
                b"no route found",
            ))),
        };

/*
        debug_assert!(
            prepare.destination().starts_with(&{
                let mut prefix = self.data.address.clone();
                prefix.extend_from_slice(&[b'.']);
                prefix
            }),
        );
        let segment_offset = self.data.address.len() + 1;
        // TODO only get the segment for multilateral routes
        let destination_segment =
            prepare.destination()[segment_offset..]
                .split(|&byte| byte == b'.')
                .next()
                .filter(|&segment| validate_address_segment(segment));
        let destination_segment = match destination_segment {
            Some(segment) => segment,
            None => return Either::B(err(self.make_reject(
                ilp::ErrorCode::F02_UNREACHABLE,
                b"invalid address segment",
            ))),
        };
*/

        let next_hop = route.endpoint(&self.data.address, prepare.destination());
        let next_hop = match next_hop {
            Ok(uri) => uri,
            Err(_error) => return Either::B(err(self.make_reject(
                ilp::ErrorCode::F02_UNREACHABLE,
                b"invalid address segment",
            ))),
        };

        let mut builder = hyper::Request::builder();
        builder.method(hyper::Method::POST);
        builder.uri(&next_hop);
        //builder.header( // TODO ILP-Peer-Name
        if let Some(auth) = route.auth() {
            builder.header(hyper::header::AUTHORIZATION, auth.clone());
        }

        std::mem::drop(routes);
        Either::A(self.client.request(builder, prepare))
    }

    fn make_reject(&self, code: ilp::ErrorCode, message: &[u8]) -> ilp::Reject {
        ilp::RejectBuilder {
            code,
            message,
            triggered_by: &self.data.address,
            data: b"",
        }.build()
    }
}

/*
#[derive(Clone, Debug)]
pub struct NextHop {
    endpoint: Uri,
    auth: Option<Bytes>,
}

impl NextHop {
    pub fn new(endpoint: Uri, auth: Option<Vec<u8>>) -> Self {
        NextHop {
            endpoint,
            auth: auth.map(Bytes::from),
        }
    }
}
*/

#[cfg(test)]
mod test_relay_service {
    use hyper::Uri;
    use lazy_static::lazy_static;

    use crate::NextHop;
    use crate::client::ClientBuilder;
    use crate::testing::{self, ADDRESS, RECEIVER_ORIGIN, ROUTES};
    use super::*;

    lazy_static! {
        static ref CLIENT: Client = ClientBuilder::new(Bytes::from(ADDRESS)).build();
        static ref RELAY: RelayService = RelayService::new(CLIENT.clone(), ROUTES.clone());
    }

    // TODO test relay to both Unilateral and Multilateral
    #[test]
    fn test_outgoing_request_unilateral() {
        testing::MockServer::new()
            .test_request(|req| {
                assert_eq!(req.method(), hyper::Method::POST);
                assert_eq!(req.uri().path(), "/alice");
                assert_eq!(
                    req.headers().get("Authorization").unwrap(),
                    "alice_auth",
                );
                assert_eq!(
                    req.headers().get("Content-Type").unwrap(),
                    "application/octet-stream",
                );
            })
            .test_body(|body| {
                assert_eq!(body.as_ref(), testing::PREPARE.as_bytes());
            })
            .with_response(|| {
                hyper::Response::builder()
                    .status(200)
                    .body(hyper::Body::from(testing::FULFILL.as_bytes()))
                    .unwrap()
            })
            .run({
                RELAY.clone()
                    .call(testing::PREPARE.clone())
                    .then(|result| -> Result<(), ()> {
                        assert_eq!(result.unwrap(), *testing::FULFILL);
                        Ok(())
                    })
            });
    }

    #[test]
    fn test_no_route() {
        let expect_reject = ilp::RejectBuilder {
            code: ilp::ErrorCode::F02_UNREACHABLE,
            message: b"no route found",
            triggered_by: ADDRESS,
            data: b"",
        }.build();
        let relay = RelayService::new(CLIENT.clone(), vec![ROUTES[1].clone()]);
        testing::MockServer::new().run({
            relay
                .call(testing::PREPARE.clone())
                .then(move |result| -> Result<(), ()> {
                    assert_eq!(result.unwrap_err(), expect_reject);
                    Ok(())
                })
        });
    }

    #[test]
    fn test_set_routes() {
        let relay = RELAY.clone();
        relay.set_routes(vec![
            Route::new(
                b"test.alice.".to_vec(),
                NextHop::Unilateral {
                    endpoint: format!("{}/new_alice", RECEIVER_ORIGIN).parse::<Uri>().unwrap(),
                    auth: None,
                },
            ),
        ]);
        testing::MockServer::new()
            .test_request(|req| {
                assert_eq!(req.uri().path(), "/new_alice");
            })
            .with_response(|| {
                hyper::Response::builder()
                    .status(200)
                    .body(hyper::Body::from(testing::FULFILL.as_bytes()))
                    .unwrap()
            })
            .run({
                relay
                    .call(testing::PREPARE.clone())
                    .then(|result| -> Result<(), ()> {
                        assert_eq!(result.unwrap(), *testing::FULFILL);
                        Ok(())
                    })
            });
    }
}
