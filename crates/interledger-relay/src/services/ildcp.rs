use std::sync::Arc;

use futures::future::{Either, FutureResult, err, ok};

use crate::{Request, Service};
use ilp::ildcp;

// TODO test

#[derive(Clone, Debug)]
pub struct ConfigService<S> {
    config: Arc<ildcp::Response>,
    //name: // TODO???
    next: S,
}

impl<S> ConfigService<S> {
    pub fn new(config: ildcp::Response, next: S) -> Self {
        ConfigService {
            config: Arc::new(config),
            next,
        }
    }
}

impl<S, Req> Service<Req> for ConfigService<S>
where
    S: Service<Req>,
    Req: RequestWithPeerName,
{
    type Future = Either<
        FutureResult<ilp::Fulfill, ilp::Reject>,
        S::Future,
    >;

    fn call(self, request: Req) -> Self::Future {
        let prepare = request.borrow(); // TODO does request.destination() work
        if prepare.destination() != ildcp::DESTINATION {
            return Either::B(self.next.call(request));
        }

        /*
        let config_request = match ildcp::Request::try_from(prepare) {
            Ok(req) => req,
            Err(_error) => return Either::A(err(ilp::RejectBuilder {
                code: ilp::ErrorCode::F00_BAD_REQUEST,
                message: b"invalid ILDCP request",
                triggered_by: &self.address,
                data: b"",
            }.build())),
        };
        */

        let peer_name = match request.peer_name() {
            Some(peer_name) => peer_name,
            None => return Either::A(err(ilp::RejectBuilder {
                code: ilp::ErrorCode::F00_BAD_REQUEST,
                message: b"Missing ILP-Peer-Name header",
                triggered_by: self.config.client_address(),
                data: b"",
            }.build())),
        };

        // TODO ILP-Peer-Name
        // TODO self.address? or self.next?
        //let fulfillment = self.next.call(request);
        //let response = ildcp::Response::try_from(fulfillment);

        let mut client_address = self.config.client_address().to_vec();
        client_address.push(b'.');
        client_address.extend_from_slice(peer_name);

        Either::A(ok(ildcp::ResponseBuilder {
            client_address: &client_address,
            asset_scale: self.config.asset_scale(),
            asset_code: self.config.asset_code(),
        }.build().into()))
    }
}

pub trait RequestWithPeerName: Request {
    fn peer_name(&self) -> Option<&[u8]>;
}
