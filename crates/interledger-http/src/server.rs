use super::HttpStore;
use bytes::{buf::Buf, Bytes, BytesMut};
use futures::{
    future::{err, Either},
    Future,
};
use interledger_packet::Prepare;
use interledger_service::{AuthToken, IncomingRequest, IncomingService};
use log::error;
use std::{
    convert::TryFrom,
    error::Error as StdError,
    fmt::{self, Display},
    net::SocketAddr,
};
use warp::{self, Filter};

/// Max message size that is allowed to transfer from a request or a message.
pub const MAX_PACKET_SIZE: u64 = 40000;

/// A warp filter that parses incoming ILP-Over-HTTP requests, validates the authorization,
/// and passes the request to an IncomingService handler.
#[derive(Clone)]
pub struct HttpServer<I, S> {
    incoming: I,
    store: S,
}

#[derive(Clone, Copy, Debug)]
enum ApiError {
    InvalidPacket,
    Unauthorized,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            ApiError::InvalidPacket => "Body was not a valid ILP Prepare packet",
            ApiError::Unauthorized => "Unauthorized",
        })
    }
}

impl StdError for ApiError {}

impl<I, S> HttpServer<I, S>
where
    I: IncomingService<S::Account> + Clone + Send + Sync + 'static,
    S: HttpStore,
{
    pub fn new(incoming: I, store: S) -> Self {
        HttpServer { incoming, store }
    }

    pub fn as_filter(
        &self,
    ) -> impl warp::Filter<Extract = (warp::http::Response<Bytes>,), Error = warp::Rejection> + Clone
    {
        let incoming = self.incoming.clone();
        let store = self.store.clone();

        warp::post2()
            .and(warp::header::<AuthToken>("authorization"))
            .and_then(move |auth: AuthToken| {
                store
                    .get_account_from_http_auth(auth.username(), auth.password())
                    .map_err(move |_| {
                        error!(
                            "Invalid authorization provided for user: {}",
                            auth.username()
                        );
                        warp::reject::custom(ApiError::Unauthorized)
                    })
            })
            .and(warp::body::content_length_limit(MAX_PACKET_SIZE))
            .and(warp::body::concat())
            .and_then(move |account: S::Account, body: warp::body::FullBody| {
                // TODO don't copy ILP packet
                let buffer = BytesMut::from(body.bytes());
                if let Ok(prepare) = Prepare::try_from(buffer) {
                    Either::A(
                        incoming
                            .clone()
                            .handle_request(IncomingRequest {
                                from: account,
                                prepare,
                            })
                            .then(|result| {
                                let bytes: BytesMut = match result {
                                    Ok(fulfill) => fulfill.into(),
                                    Err(reject) => reject.into(),
                                };
                                Ok(warp::http::Response::builder()
                                    .header("Content-Type", "application/octet-stream")
                                    .status(200)
                                    .body(bytes.freeze())
                                    .unwrap())
                            }),
                    )
                } else {
                    error!("Body was not a valid Prepare packet");
                    Either::B(err(warp::reject::custom(ApiError::InvalidPacket)))
                }
            })
    }

    pub fn bind(&self, addr: SocketAddr) -> impl Future<Item = (), Error = ()> + Send {
        warp::serve(self.as_filter()).bind(addr)
    }
}
