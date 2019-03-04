use bytes::{BufMut, Bytes, BytesMut};
use http::HttpTryFrom;
use http::uri::InvalidUriBytes;
use hyper::Uri;

/// A simple static routing table.
#[derive(Debug, PartialEq)]
pub struct RoutingTable(Vec<Route>);

// TODO validate target prefix? "^[a-zA-Z0-9._~-]+$"

impl RoutingTable {
    #[inline]
    pub fn new(routes: Vec<Route>) -> Self {
        RoutingTable(routes)
    }

    pub fn resolve(&self, destination: &[u8]) -> Option<&Route> {
        self.0
            .iter()
            .find(|route| {
                destination.starts_with(&route.target_prefix)
            })
    }
}

impl Default for RoutingTable {
    fn default() -> Self {
        RoutingTable::new(Vec::new())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    target_prefix: Vec<u8>,
    next_hop: NextHop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NextHop {
    Unilateral {
        endpoint: Uri,
        auth: Option<Bytes>,
    },
    Multilateral {
        uri_prefix: Bytes,
        uri_suffix: Bytes,
        auth: Option<Bytes>,
    },
}

impl Route {
    #[inline]
    pub fn new(
        target_prefix: Vec<u8>,
        next_hop: NextHop,
    ) -> Self {
        Route { target_prefix, next_hop }
    }

    #[inline]
    pub fn target_prefix(&self) -> &[u8] {
        &self.target_prefix[..]
    }

    pub fn endpoint(&self, connector_addr: &[u8], destination_addr: &[u8])
        -> Result<Uri, RouterError>
    {
        match &self.next_hop {
            // `hyper::Uri` is built from `bytes::Bytes`, so this clone doesn't
            // actually allocate.
            NextHop::Unilateral { endpoint, .. } => Ok(endpoint.clone()),
            NextHop::Multilateral { uri_prefix, uri_suffix, .. } => {
                // TODO or use the route's target prefix instead of the connector address?
                let destination_segment = match parse_address_segment(
                    connector_addr,
                    destination_addr,
                ) {
                    Some(segment) => segment,
                    None => return Err(RouterError(ErrorKind::InvalidDestination)),
                };

                // TODO dont allocate every time (maybe have a cache of segment => uri)
                let mut uri = BytesMut::with_capacity({
                    uri_prefix.len()
                    + destination_segment.len()
                    + uri_suffix.len()
                });
                uri.put_slice(uri_prefix);
                uri.put_slice(destination_segment);
                uri.put_slice(uri_suffix);
                Ok(Uri::try_from(uri.freeze())?)
            },
        }
    }

    pub fn auth(&self) -> Option<&Bytes> {
        match &self.next_hop {
            NextHop::Unilateral { auth, .. } => auth.as_ref(),
            NextHop::Multilateral { auth, .. } => auth.as_ref(),
        }
    }
}

#[derive(Debug)]
pub struct RouterError(ErrorKind);

#[derive(Debug)]
enum ErrorKind {
    InvalidDestination,
    InvalidUriBytes(InvalidUriBytes),
}

impl From<InvalidUriBytes> for RouterError {
    fn from(inner: InvalidUriBytes) -> Self {
        RouterError(ErrorKind::InvalidUriBytes(inner))
    }
}

fn parse_address_segment<'a>(connector: &[u8], destination: &'a [u8])
    -> Option<&'a [u8]>
{
    debug_assert!(
        destination.starts_with(connector)
            && destination[connector.len()] == b'.',
    );
    let segment_offset = connector.len() + 1;
    destination[segment_offset..]
        .split(|&byte| byte == b'.')
        .next()
        .filter(|&segment| validate_address_segment(segment))
}

fn validate_address_segment(segment: &[u8]) -> bool {
    !segment.is_empty() && segment.iter().all(|&byte| {
        byte == b'-' || byte == b'_'
            || (b'A' <= byte && byte <= b'Z')
            || (b'a' <= byte && byte <= b'z')
            || (b'0' <= byte && byte <= b'9')
    })
}

#[cfg(test)]
mod test_routing_table {
    use lazy_static::lazy_static;

    use crate::testing::ROUTES;
    use super::*;

    lazy_static! {
        static ref HOP_0: NextHop = ROUTES[0].next_hop.clone();
        static ref HOP_1: NextHop = ROUTES[1].next_hop.clone();
        static ref HOP_2: NextHop = ROUTES[2].next_hop.clone();
    }

    #[test]
    fn test_resolve() {
        let table = RoutingTable::new(vec![
            Route::new(b"test.one".to_vec(), HOP_0.clone()),
            Route::new(b"test.two".to_vec(), HOP_1.clone()),
            Route::new(b"test.".to_vec(), HOP_2.clone()),
        ]);
        let routes = &table.0;
        // Exact match.
        assert_eq!(table.resolve(b"test.one"), Some(&routes[0]));
        // Prefix match.
        assert_eq!(table.resolve(b"test.one.alice"), Some(&routes[0]));
        assert_eq!(table.resolve(b"test.two.bob"), Some(&routes[1]));
        assert_eq!(table.resolve(b"test.three"), Some(&routes[2]));
        // Dot separator isn't necessary.
        assert_eq!(table.resolve(b"test.two__"), Some(&routes[1]));
        // No matching prefix.
        assert_eq!(table.resolve(b"example.test.one"), None);
        assert_eq!(table.resolve(b""), None);
    }

    #[test]
    fn test_resolve_catch_all() {
        let table = RoutingTable::new(vec![
            Route::new(b"test.one".to_vec(), HOP_0.clone()),
            Route::new(b"test.two".to_vec(), HOP_1.clone()),
            Route::new(b"".to_vec(), HOP_2.clone()),
        ]);
        assert_eq!(table.resolve(b"example.test.one"), Some(&table.0[2]));
    }
}

#[cfg(test)]
mod test_route {
    use lazy_static::lazy_static;

    use super::*;

    lazy_static! {
        static ref UNI_URI: Uri =
            "http://example.com/alice".parse::<Uri>().unwrap();

        static ref UNI: Route = Route::new(
            b"test.alice.".to_vec(),
            NextHop::Unilateral {
                endpoint: UNI_URI.clone(),
                auth: Some(Bytes::from("alice_auth")),
            },
        );

        static ref MULTI: Route = Route::new(
            b"test.relay.".to_vec(),
            NextHop::Multilateral {
                uri_prefix: Bytes::from("http://example.com/bob/"),
                uri_suffix: Bytes::from("/ilp"),
                auth: Some(Bytes::from("bob_auth")),
            },
        );
    }

    #[test]
    fn test_target_prefix() {
        assert_eq!(UNI.target_prefix(), b"test.alice.");
    }

    #[test]
    fn test_endpoint() {
        assert_eq!(
            UNI.endpoint(b"test.relay", b"test.whatever.123").unwrap(),
            *UNI_URI,
        );
        assert_eq!(
            MULTI.endpoint(b"test.relay", b"test.relay.123.456").unwrap(),
            "http://example.com/bob/123/ilp".parse::<Uri>().unwrap(),
        );
        assert!(MULTI.endpoint(b"test.relay", b"test.relay.123!.456").is_err());
    }

    #[test]
    fn test_auth() {
        assert_eq!(UNI.auth(), Some(&Bytes::from("alice_auth")));
        assert_eq!(MULTI.auth(), Some(&Bytes::from("bob_auth")));
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    #[test]
    fn test_parse_address_segment() {
        assert_eq!(
            parse_address_segment(b"test.cx", b"test.cx.alice"),
            Some(&b"alice"[..]),
        );
        assert_eq!(
            parse_address_segment(b"test.cx", b"test.cx.alice.bob"),
            Some(&b"alice"[..]),
        );
        assert_eq!(
            parse_address_segment(b"test.cx", b"test.cx.alice!"),
            None,
        );
    }

    #[test]
    fn test_validate_address_segment() {
        let valid = &["test", "Test_1-2-3"];
        let invalid = &["", "peer.config", "test!", "test?"];

        for segment in valid {
            assert_eq!(validate_address_segment(segment.as_bytes()), true);
        }
        for segment in invalid {
            assert_eq!(validate_address_segment(segment.as_bytes()), false);
        }
    }
}
