//use std::error::Error as StdError;
//use std::net;
use std::time;

use bytes::Bytes;
use futures::future::{Either, ok};
use futures::prelude::*;
use hyper::Uri;

use crate::{ClientBuilder, Route};
use crate::middlewares::{AuthToken, AuthTokenFilter, MethodFilter, Receiver};
use crate::services::{ConfigService, ExpiryService, RelayService};
use ilp::ildcp;

// TODO
/// The maximum duration that the outgoing HTTP client will wait for a response,
/// even if the Prepare's expiry is longer.
const DEFAULT_MAX_TIMEOUT: time::Duration = time::Duration::from_secs(60);

#[derive(Debug)]
pub struct ConnectorBuilder {
    //pub name: Vec<u8>, // TODO or in the route?
    // TODO rename field, to avoid address.address
    pub address: ConnectorAddress,
    //pub relation: ConnectorRelation,
    //pub ilp_addr: Vec<u8>,
    pub auth_tokens: Vec<AuthToken>,
    pub routes: Vec<Route>,
}

/* TODO
pub struct Config {
    pub net_addr: net::SocketAddr,
    pub ilp_addr: Vec<u8>,
    pub routes: Vec<Route<hyper::Uri>>,
}
*/

// TODO test dynamic and static
#[derive(Debug)]
pub enum ConnectorAddress {
    Static {
        address: Vec<u8>,
        asset_scale: u8,
        asset_code: Vec<u8>,
        //address: Bytes,
    },
    Dynamic {
        parent_endpoint: Uri,
        parent_auth: Vec<u8>,
        name: Vec<u8>, // TODO should this be optional?
    },
}

type Connector =
    MethodFilter<AuthTokenFilter<
        Receiver<
            ExpiryService<ConfigService<RelayService>>,
        >,
    >>;

impl ConnectorBuilder {
    pub fn build(self) -> impl Future<
        Item = Connector,
        //Error = ilp::ParseError,
        // TODO use an actual error
        Error = (),
    > {
        self.address.load_config().map(move |ildcp| {
            let address = Bytes::from(ildcp.client_address());

            let client = ClientBuilder::new(address.clone()).build();
            let relay_svc = RelayService::new(client, self.routes);
            let ildcp_svc = ConfigService::new(ildcp, relay_svc);
            let expiry_svc = ExpiryService::new(address, DEFAULT_MAX_TIMEOUT, ildcp_svc);

            let receiver = Receiver::new(expiry_svc);
            let auth_filter = AuthTokenFilter::new(self.auth_tokens, receiver);
            MethodFilter::new(hyper::Method::POST, auth_filter)
        })
    }
}

impl ConnectorAddress {
    // TODO maybe return &[u8]
    fn load_config(&self)
        -> impl Future<Item = ildcp::Response, Error = ()>
    {
        use ConnectorAddress::*;
        match self {
            Static {
                address,
                asset_code,
                asset_scale,
            } => Either::A(ok(ildcp::ResponseBuilder {
                client_address: &address,
                asset_code: &asset_code,
                asset_scale: *asset_scale,
            }.build())),
            // TODO ildcp::fetch()
            Dynamic {
                parent_endpoint,
                parent_auth,
                name,
            } => Either::B({
                let client = ClientBuilder::new(Bytes::new()).build();
                let mut request = hyper::Request::builder();
                request.method(hyper::Method::POST);
                request.uri(parent_endpoint);
                request.header(hyper::header::AUTHORIZATION, &parent_auth[..]);
                request.header("ILP-Peer-Name", &name[..]);

                let prepare = ildcp::Request::new().to_prepare();
                client.request(request, prepare)
                    .then(|result| {
                        // XXX dont unwrap
                        let fulfill = result.unwrap();
                        ok(ildcp::Response::try_from(fulfill)
                            .unwrap()) // XXX dont unwrap
                            //.map(|response| Bytes::from(response.client_address()))
                            //.map_err(|error| Box::new(error))
                    })
                    //.then(|result| match result {
                    //    Ok(fulfill) => {
                    //        ildcp::Response::try_from(fulfill)
                    //            .map_err(|error| Box::new(error))
                    //    },
                    //    Err(reject) => Err(Box::new(format!(
                    //        "error fetching address from parent code={} message={:?}",
                    //        reject.code(), reject.message(),
                    //    ))),
                    //})
                    //.map(|response| response.client_address())
            }),
        }
    }
}

#[cfg(test)]
mod test_connector_builder {
    use futures::prelude::*;

    use crate::testing::{self, FULFILL, PREPARE};
    use super::*;

    static CONNECTOR_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 3002);

    #[test]
    fn test_relay() {
        let start_connector = ConnectorBuilder {
            //ilp_addr: b"example.alice".to_vec(),
            address: ConnectorAddress::Static {
                address: b"example.alice".to_vec(),
                asset_scale: 9,
                asset_code: b"XRP".to_vec(),
            },
            auth_tokens: vec![AuthToken::new(b"secret".to_vec())],
            routes: testing::ROUTES.clone(),
        }.build();

        let request = hyper::Client::new()
            .request({
                hyper::Request::post("http://127.0.0.1:3002/ilp")
                    .header("Authorization", "secret")
                    .body(hyper::Body::from(PREPARE.as_bytes()))
                    .unwrap()
            })
            .and_then(|response| {
                assert_eq!(response.status(), 200);
                response.into_body().concat2()
            })
            .map(|body| {
                assert_eq!(body.as_ref(), FULFILL.as_bytes());
            });

        let start_server = start_connector.and_then(|connector| {
            hyper::Server::bind(&CONNECTOR_ADDR.into())
                .serve(move || -> Result<_, &'static str> {
                    Ok(connector.clone())
                })
                .with_graceful_shutdown(request)
                .map_err(|err| panic!(err))
        });

        testing::MockServer::new()
            .test_request(|req| {
                assert_eq!(req.method(), hyper::Method::POST);
                assert_eq!(req.uri().path(), "/alice");
                assert_eq!(
                    req.headers().get("Content-Type").unwrap(),
                    "application/octet-stream",
                );
            })
            .test_body(|body| {
                assert_eq!(body.as_ref(), PREPARE.as_bytes());
            })
            .with_response(|| {
                hyper::Response::builder()
                    .status(200)
                    .body(hyper::Body::from(FULFILL.as_bytes()))
                    .unwrap()
            })
            .run(start_server);
    }
}
