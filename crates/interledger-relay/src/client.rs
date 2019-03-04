use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use futures::future::{Either, err};
use futures::prelude::*;
use http::request::Builder as RequestBuilder;
use hyper::{Response, StatusCode};

type HyperClient = hyper::Client<hyper::client::HttpConnector, hyper::Body>;

static OCTET_STREAM: &'static [u8] = b"application/octet-stream";

// TODO remove builder?
#[derive(Clone, Debug)]
pub struct Client {
    address: Bytes,
    hyper: Arc<HyperClient>,
}

#[derive(Clone, Debug)]
pub struct ClientBuilder {
    address: Bytes,
    client: Option<HyperClient>,
}

impl Client {
    pub fn address(&self) -> &Bytes {
        &self.address
    }

    /// `req_builder` is the request template. At a minimum, it should have the
    /// URI and method set.
    pub fn request(self, mut req_builder: RequestBuilder, prepare: ilp::Prepare)
        -> impl Future<Item = ilp::Fulfill, Error = ilp::Reject>
    {
        let prepare_bytes = BytesMut::from(prepare).freeze();
        let req = req_builder
            .header(hyper::header::CONTENT_TYPE, OCTET_STREAM)
            .body(hyper::Body::from(prepare_bytes))
            .expect("build_prepare_request builder error");

        self.hyper
            .request(req)
            .then(move |res| match res {
                Ok(res) => Either::A(self.decode_http_response(res)),
                Err(_error) => Either::B(err(self.make_reject(
                    ilp::ErrorCode::T01_PEER_UNREACHABLE,
                    b"peer connection error",
                ))),
            })
    }

    fn decode_http_response(self, res: Response<hyper::Body>)
        -> impl Future<Item = ilp::Fulfill, Error = ilp::Reject>
    {
        let status = res.status();
        if status == StatusCode::OK {
            Either::A(res.into_body()
                .concat2()
                .then(move |body| {
                    match body {
                        Ok(body) => {
                            let body = BytesMut::from(Bytes::from(body));
                            self.decode_response(body).into_future()
                        },
                        Err(_error) => err(self.make_reject(
                            ilp::ErrorCode::T00_INTERNAL_ERROR,
                            b"invalid response body from peer",
                        )),
                    }
                }))
        } else if status.is_client_error() {
            Either::B(err(self.make_reject(
                ilp::ErrorCode::F00_BAD_REQUEST,
                b"bad request to peer",
            )))
        } else if status.is_server_error() {
            Either::B(err(self.make_reject(
                ilp::ErrorCode::T01_PEER_UNREACHABLE,
                b"peer internal error",
            )))
        } else {
            Either::B(err(self.make_reject(
                ilp::ErrorCode::T00_INTERNAL_ERROR,
                b"unexpected response code from peer",
            )))
        }
    }

    fn decode_response(&self, bytes: BytesMut)
        -> Result<ilp::Fulfill, ilp::Reject>
    {
        match ilp::Packet::try_from(bytes) {
            Ok(ilp::Packet::Fulfill(fulfill)) => Ok(fulfill),
            Ok(ilp::Packet::Reject(reject)) => Err(reject),
            _ => Err(self.make_reject(
                ilp::ErrorCode::T00_INTERNAL_ERROR,
                b"invalid response body from peer",
            )),
        }
    }

    fn make_reject(&self, code: ilp::ErrorCode, message: &[u8]) -> ilp::Reject {
        ilp::RejectBuilder {
            code,
            message,
            triggered_by: &self.address,
            data: b"",
        }.build()
    }
}

impl ClientBuilder {
    pub fn new(address: Bytes) -> Self {
        ClientBuilder {
            address,
            client: None,
        }
    }

    pub fn build(self) -> Client {
        Client {
            address: self.address,
            hyper: Arc::new(self.client.unwrap_or_else(HyperClient::new)),
        }
    }

    pub fn with_client(mut self, client: HyperClient) -> Self {
        self.client = Some(client);
        self
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use crate::testing::{self, RECEIVER_ORIGIN};
    use super::*;

    static ADDRESS: &'static [u8] = b"example.connector";

    lazy_static! {
        static ref CLIENT: Client = ClientBuilder::new(Bytes::from(ADDRESS))
            .build();

        static ref CLIENT_HTTP2: Client = ClientBuilder::new(Bytes::from(ADDRESS))
            .with_client(
                hyper::Client::builder()
                    .http2_only(true)
                    .build_http(),
            )
            .build();
    }

    fn make_request() -> RequestBuilder {
        let mut builder = hyper::Request::builder();
        builder.method(hyper::Method::POST);
        builder.uri(RECEIVER_ORIGIN);
        builder.header("Authorization", "alice_auth");
        builder
    }

    #[test]
    fn test_outgoing_request() {
        testing::MockServer::new()
            .test_request(|req| {
                assert_eq!(req.method(), hyper::Method::POST);
                assert_eq!(req.uri().path(), "/");
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
                CLIENT.clone()
                    .request(make_request(), testing::PREPARE.clone())
                    .then(|result| -> Result<(), ()> {
                        assert_eq!(result.unwrap(), *testing::FULFILL);
                        Ok(())
                    })
            });
    }

    #[test]
    fn test_outgoing_http2_only() {
        testing::MockServer::new()
            .test_request(|req| {
                assert_eq!(req.version(), hyper::Version::HTTP_2);
            })
            .with_response(|| {
                hyper::Response::builder()
                    .status(200)
                    .body(hyper::Body::from(testing::FULFILL.as_bytes()))
                    .unwrap()
            })
            .run({
                CLIENT_HTTP2.clone()
                    .request(make_request(), testing::PREPARE.clone())
                    .then(|result| -> Result<(), ()> {
                        assert_eq!(result.unwrap(), *testing::FULFILL);
                        Ok(())
                    })
            });
    }

    #[test]
    fn test_incoming_reject() {
        testing::MockServer::new()
            .with_response(|| {
                hyper::Response::builder()
                    .status(200)
                    .body(hyper::Body::from(testing::REJECT.as_bytes()))
                    .unwrap()
            })
            .run({
                CLIENT.clone()
                    .request(make_request(), testing::PREPARE.clone())
                    .then(|result| -> Result<(), ()> {
                        assert_eq!(result.unwrap_err(), *testing::REJECT);
                        Ok(())
                    })
            });
    }

    #[test]
    fn test_incoming_invalid_packet() {
        let expect_reject = ilp::RejectBuilder {
            code: ilp::ErrorCode::T00_INTERNAL_ERROR,
            message: b"invalid response body from peer",
            triggered_by: ADDRESS,
            data: b"",
        }.build();
        testing::MockServer::new()
            .with_response(|| {
                hyper::Response::builder()
                    .status(200)
                    .body(hyper::Body::from(&b"this is not a packet"[..]))
                    .unwrap()
            })
            .run({
                CLIENT.clone()
                    .request(make_request(), testing::PREPARE.clone())
                    .then(move |result| -> Result<(), ()> {
                        assert_eq!(result.unwrap_err(), expect_reject);
                        Ok(())
                    })
            });
    }

    macro_rules! make_test_incoming_error_code {
        (
            fn $fn:ident(
                status_code: $status_code:expr,
                error_code: $error_code:expr,
                error_message: $error_message:expr $(,)?
            );
        ) => {
            #[test]
            fn $fn() {
                let expect_reject = ilp::RejectBuilder {
                    code: $error_code,
                    message: $error_message,
                    triggered_by: ADDRESS,
                    data: b"",
                }.build();
                testing::MockServer::new()
                    .with_response(|| {
                        hyper::Response::builder()
                            .status($status_code)
                            .body(hyper::Body::from(testing::FULFILL.as_bytes()))
                            .unwrap()
                    })
                    .run({
                        CLIENT.clone()
                            .request(make_request(), testing::PREPARE.clone())
                            .then(move |result| -> Result<(), ()> {
                                assert_eq!(result.unwrap_err(), expect_reject);
                                Ok(())
                            })
                    });
            }
        };
    }

    make_test_incoming_error_code! {
        fn test_incoming_300(
            status_code: 300,
            error_code: ilp::ErrorCode::T00_INTERNAL_ERROR,
            error_message: b"unexpected response code from peer",
        );
    }

    make_test_incoming_error_code! {
        fn test_incoming_400(
            status_code: 400,
            error_code: ilp::ErrorCode::F00_BAD_REQUEST,
            error_message: b"bad request to peer",
        );
    }

    make_test_incoming_error_code! {
        fn test_incoming_500(
            status_code: 500,
            error_code: ilp::ErrorCode::T01_PEER_UNREACHABLE,
            error_message: b"peer internal error",
        );
    }

    #[test]
    fn test_incoming_abort() {
        let expect_reject = ilp::RejectBuilder {
            code: ilp::ErrorCode::T01_PEER_UNREACHABLE,
            message: b"peer connection error",
            triggered_by: ADDRESS,
            data: b"",
        }.build();
        testing::MockServer::new()
            .with_abort()
            .run({
                CLIENT.clone()
                    .request(make_request(), testing::PREPARE.clone())
                    .then(move |result| -> Result<(), ()> {
                        assert_eq!(result.unwrap_err(), expect_reject);
                        Ok(())
                    })
            });
    }
}
