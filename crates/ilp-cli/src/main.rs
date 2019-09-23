use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use reqwest;
use std::{collections::HashMap, process::exit};

pub fn main() {
    // Define the arguments to the CLI
    let mut app = App::new("ilp-cli")
        .about("Interledger.rs Command-Line Interface")
        .version(crate_version!())
        // TODO remove this line once this issue is solved:
        // https://github.com/clap-rs/clap/issues/1536
        .after_help("")
        .args(&[
            Arg::with_name("authorization_key")
                .long("auth")
                .env("ILP_CLI_AUTH")
                .required(true)
                .help("An authorization key granting access to the designated operation"),
            Arg::with_name("node_address")
                .long("node")
                .env("ILP_CLI_NODE")
                .default_value("localhost:7770")
                .help("The URL of the node to which to connect"),
            Arg::with_name("print_response")
                .short("p")
                .long("print-response")
                .help("Upon a successful HTTP response, response body will be printed to stdout"),
        ])
        .subcommands(vec![
            // Example: ilp-cli add-account alice ABC 9
            SubCommand::with_name("add-account")
                .about("Creates a new account on this node")
                .args(&[
                    // Required, positional arguments
                    Arg::with_name("username")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .help("The username of the new account"),
                    Arg::with_name("asset_code")
                        .index(2)
                        .takes_value(true)
                        .required(true)
                        .help("The code of the asset associated with this account"),
                    Arg::with_name("asset_scale")
                        .index(3)
                        .takes_value(true)
                        .required(true)
                        .help("The scale of the asset associated with this account"),
                    // Optional, named arguments
                    Arg::with_name("ilp_address").long("ilp-address").takes_value(true),
                    Arg::with_name("max_packet_amount").long("max-packet-amount").takes_value(true),
                    Arg::with_name("min_balance").long("min-balance").takes_value(true).allow_hyphen_values(true),
                    Arg::with_name("ilp_over_http_url").long("ilp-over-http-url").takes_value(true),
                    Arg::with_name("ilp_over_http_incoming_token").long("ilp-over-http-incoming-token").takes_value(true),
                    Arg::with_name("ilp_over_http_outgoing_token").long("ilp-over-http-outgoing-token").takes_value(true),
                    Arg::with_name("ilp_over_btp_url").long("ilp-over-btp-url").takes_value(true),
                    Arg::with_name("ilp_over_btp_outgoing_token").long("ilp-over-btp-outgoing-token").takes_value(true),
                    Arg::with_name("ilp_over_btp_incoming_token").long("ilp-over-btp-incoming-token").takes_value(true),
                    Arg::with_name("settle_threshold").long("settle-threshold").takes_value(true).allow_hyphen_values(true),
                    Arg::with_name("settle_to").long("settle-to").takes_value(true).allow_hyphen_values(true),
                    Arg::with_name("routing_relation").long("routing-relation").takes_value(true),
                    Arg::with_name("round_trip_time").long("round-trip-time").takes_value(true),
                    Arg::with_name("amount_per_minute_limit").long("amount-per-minute-limit").takes_value(true),
                    Arg::with_name("packets_per_minute_limit").long("packets-per-minute-limit").takes_value(true),
                    Arg::with_name("settlement_engine_url").long("settlement-engine-url").takes_value(true),
                ]),
            // Example: ilp-cli get-balance alice
            SubCommand::with_name("get-balance")
                .about("Returns the balance of an account")
                .arg(Arg::with_name("account_username")
                    .index(1)
                    .takes_value(true)
                    .required(true)
                    .help("The username of the account whose balance to return")),
            // Example: ilp-cli post-payment alice 500 "http://localhost:8770/accounts/bob/spsp"
            SubCommand::with_name("post-payment")
                .about("Issue a payment from an account on this node")
                .args(&[
                    Arg::with_name("sender_username")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .help("The username of the account on this node issuing the payment"),
                    Arg::with_name("source_amount")
                        .index(2)
                        .takes_value(true)
                        .required(true)
                        .help("The amount to transfer from the sender to the receiver, denominated in units of the sender's assets"),
                    Arg::with_name("receiver")
                        .index(3)
                        .takes_value(true)
                        .required(true)
                        // TODO: better way of describing this parameter
                        .help("The SPSP address of the account receiving the payment"),
                ]),
        ]);

    // Parse the CLI input using the defined arguments
    let matches = app.clone().get_matches();

    // `--auth` is a required argument, so will never be None
    let auth = matches.value_of("authorization_key").unwrap();
    // `--node` has a a default valiue, so will never be None
    let node = matches.value_of("node_address").unwrap();

    // Dispatch based on parsed input
    match matches.subcommand() {
        // Execute the specified subcommand
        (subcommand_name, Some(subcommand_matches)) => {
            // Send HTTP request
            let client = reqwest::Client::new();
            let response = match subcommand_name {
                "add-account" => {
                    let args = extract_args(subcommand_matches);
                    client
                        // TODO: tacking on the protocol like this doesn't feel ideal,
                        // should we require it to be specified as part of the argument?
                        // We could also find a way to legitimately build a URL from parts
                        // rather than merely interpolating a string.
                        .post(&format!("http://{}/accounts", node))
                        .bearer_auth(auth)
                        .json(&args)
                        .send()
                }
                "get-balance" => {
                    let user = subcommand_matches.value_of("account_username").unwrap();
                    client
                        .get(&format!("http://{}/accounts/{}/balance", node, user))
                        .bearer_auth(auth)
                        .send()
                }
                "post-payment" => {
                    let mut args = extract_args(subcommand_matches);
                    let user = args.remove("sender_username").unwrap();
                    client
                        .post(&format!("http://{}/accounts/{}/payments", node, user))
                        .bearer_auth(&format!("{}:{}", user, auth))
                        .json(&args)
                        .send()
                }
                name => panic!("Unhandled subcommand: {}", name),
            };

            // Handle HTTP response
            match response {
                Err(e) => {
                    eprintln!("ILP CLI error: failed to send request: {}", e);
                    exit(1);
                }
                Ok(mut res) => match res.text() {
                    Err(e) => {
                        eprintln!("ILP CLI error: failed to parse response: {}", e);
                        exit(1);
                    }
                    // Final output
                    Ok(val) => {
                        if res.status().is_success() {
                            if matches.is_present("print_response") {
                                println!("{}", val)
                            }
                        } else {
                            eprintln!(
                                "ILP CLI error: unsuccessful response from node: {}: {}",
                                res.status(),
                                val
                            );
                            exit(1);
                        }
                    }
                },
            }
        }
        // No subcommand identified within parsed input
        _ => app.print_help().unwrap(),
    }
}

// This function takes the map of arguments parsed by Clap
// and extracts the values for each argument.
fn extract_args<'a>(matches: &'a ArgMatches) -> HashMap<&'a str, &'a str> {
    matches // Contains data and metadata about the parsed command
        .args // The hashmap containing each parameter along with its values and metadata
        .iter()
        .map(|(&key, val)| (key, val.vals[0].to_str().unwrap())) // Extract raw key/value pairs
        .collect()
}
