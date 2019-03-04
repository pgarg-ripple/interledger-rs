use futures::prelude::*;
use hyper::Uri;

use interledger_relay::{AuthToken, NextHop, Route};
use interledger_relay::app::{ConnectorAddress, ConnectorBuilder};

//struct Config (maybe in another module)

fn main() {
    // TODO config from json
    let start_connector = ConnectorBuilder {
        //net_addr: ...
        //ilp_addr: b"example.alice".to_vec(),
        address: ConnectorAddress::Static {
            address: b"example.alice".to_vec(),
            asset_scale: 9,
            asset_code: b"XRP".to_vec(),
        },
        auth_tokens: vec![
            AuthToken::new(b"secret".to_vec()),
        ],
        routes: vec![
            Route::new(
                b"".to_vec(),
                NextHop::Unilateral {
                    endpoint: "http://127.0.0.1:3002/ilp".parse::<Uri>().unwrap(),
                    auth: None,
                },
            ),
        ],
    }.build();

    //let start_connector = start_connector
    //    .map_err(|error| {
    //        panic!(format!("error starting connector: {}", error));
    //    });

    let run_server = start_connector
        .and_then(|connector| {
            hyper::Server::bind(&([127, 0, 0, 1], 3001).into())
                // NOTE: `hyper::Error` is a placeholder.. The "never" type would
                // be better once it's stable.
                .serve(move || -> Result<_, hyper::Error> {
                    Ok(connector.clone())
                })
                .map_err(|error| {
                    eprintln!("server error: {}", error)
                })
        });
        // XXX?
        //.map_err(|error| {
        //    eprintln!("server error: {}", error);
        //});

    hyper::rt::run(run_server);
}
