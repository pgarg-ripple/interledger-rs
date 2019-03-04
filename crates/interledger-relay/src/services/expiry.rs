use std::cmp;
use std::time;

use bytes::Bytes;
use futures::future::err;
use futures::prelude::*;
use tokio::util::FutureExt;

use crate::{Request, Service};

/// Reject expired Prepares, and time out requests that take too long.
#[derive(Clone, Debug)]
pub struct ExpiryService<S> {
    address: Bytes,
    max_timeout: time::Duration,
    next: S,
}

impl<S> ExpiryService<S> {
    pub fn new(
        address: Bytes,
        max_timeout: time::Duration,
        next: S,
    ) -> Self {
        ExpiryService { address, max_timeout, next }
    }

    fn make_reject(&self, code: ilp::ErrorCode, message: &[u8])
        -> ilp::Reject
    {
        ilp::RejectBuilder {
            code,
            message,
            triggered_by: &self.address,
            data: &[],
        }.build()
    }
}

impl<S, Req> Service<Req> for ExpiryService<S>
where
    S: Service<Req> + Send + 'static,
    Req: Request,
{
    /*type Future = Either<
        Timeout<S::Future>,
        FutureResult<ilp::Fulfill, ilp::Reject>,
    >;*/
    type Future = Box<
        dyn Future<
            Item = ilp::Fulfill,
            Error = ilp::Reject,
        > + Send + 'static,
    >;

    fn call(self, request: Req) -> Self::Future {
        let prepare = request.borrow();
        let expires_at = prepare.expires_at();
        let expires_in = expires_at.duration_since(time::SystemTime::now());

        let expires_in = match expires_in {
            Ok(expires_in) => expires_in,
            Err(_) => return Box::new(err(self.make_reject(
                ilp::ErrorCode::R02_INSUFFICIENT_TIMEOUT,
                b"insufficient timeout",
            ))),
        };

        let next = self.next.clone();
        Box::new(next
            .call(request)
            .timeout(cmp::min(self.max_timeout, expires_in))
            .map_err(move |error| match (error.is_elapsed(), error.into_inner()) {
                (_, Some(reject)) => reject,
                (true, None) => self.make_reject(
                    ilp::ErrorCode::R00_TRANSFER_TIMED_OUT,
                    b"request timed out",
                ),
                (false, None) => self.make_reject(
                    ilp::ErrorCode::T00_INTERNAL_ERROR,
                    b"timer error",
                ),
            })
        )
    }
}

#[cfg(test)]
mod test_expiry_service {
    use futures::future::ok;
    use lazy_static::lazy_static;

    use crate::testing::{DelayService, FULFILL, MockService, PanicService, PREPARE};
    use super::*;

    lazy_static! {
        static ref ADDRESS: Bytes = Bytes::from(&b"test.alice"[..]);
    }

    const MAX_TIMEOUT: time::Duration = time::Duration::from_secs(60);

    #[test]
    fn test_ok() {
        let receiver = MockService::new(Ok(FULFILL.clone()));
        let expiry = ExpiryService::new(ADDRESS.clone(), MAX_TIMEOUT, receiver);

        let future = expiry.call(PREPARE.clone())
            .then(|response| {
                assert_eq!(response, Ok(FULFILL.clone()));
                ok(())
            });
        tokio::run(future);
    }

    #[test]
    fn test_insufficient_timeout() {
        let mut prepare = PREPARE.clone();
        prepare.set_expires_at(time::SystemTime::now());

        let receiver = PanicService;
        let expiry = ExpiryService::new(ADDRESS.clone(), MAX_TIMEOUT, receiver);

        let future = expiry.call(prepare)
            .then(|response| {
                let reject = response.unwrap_err();
                assert_eq!(reject.code(), ilp::ErrorCode::R02_INSUFFICIENT_TIMEOUT);
                assert_eq!(reject.message(), b"insufficient timeout");
                ok(())
            });
        tokio::run(future);
    }

    #[test]
    fn test_timed_out() {
        const SOON: time::Duration = time::Duration::from_millis(100);
        let mut prepare = PREPARE.clone();
        prepare.set_expires_at(time::SystemTime::now() + SOON);

        let receiver = MockService::new(Ok(FULFILL.clone()));
        let receiver = DelayService::new(
            SOON + time::Duration::from_millis(1),
            receiver,
        );
        let expiry = ExpiryService::new(ADDRESS.clone(), MAX_TIMEOUT, receiver);

        let future = expiry.call(prepare)
            .then(|response| {
                let reject = response.unwrap_err();
                assert_eq!(reject.code(), ilp::ErrorCode::R00_TRANSFER_TIMED_OUT);
                assert_eq!(reject.message(), b"request timed out");
                ok(())
            });
        tokio::run(future);
    }

    #[test]
    fn test_max_timeout() {
        const MAX_TIMEOUT: time::Duration = time::Duration::from_millis(15);
        let receiver = MockService::new(Ok(FULFILL.clone()));
        let receiver = DelayService::new(time::Duration::from_millis(20), receiver);
        let expiry = ExpiryService::new(ADDRESS.clone(), MAX_TIMEOUT, receiver);

        let future = expiry.call(PREPARE.clone())
            .then(|response| {
                let reject = response.unwrap_err();
                assert_eq!(reject.code(), ilp::ErrorCode::R00_TRANSFER_TIMED_OUT);
                assert_eq!(reject.message(), b"request timed out");
                ok(())
            });
        tokio::run(future);
    }
}
