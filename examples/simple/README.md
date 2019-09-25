# Simple Two-Node Interledger Payment
> A demo of sending a payment between 2 Interledger.rs nodes without settlement.

## Overview

This example sets up two local Interledger.rs nodes, peers them together, and sends a payment from one to the other. This example does not involve any remote nodes or networks. 

To run the full example, you can use [`run-md.sh`](../../scripts/run-md.sh) as described [here](../README.md). Otherwise, you can walk through each step below.

Each of the services write their logs to files found under the `logs` directory. You can run `tail -f logs/node_a.log`, for example, to watch the logs of Node A.

![overview](images/overview.svg)

## Prerequisites

- [Rust](#rust)
- [Redis](#redis)

### Rust

Because Interledger.rs is written in the Rust language, you need the Rust environment. Refer to the [Getting started](https://www.rust-lang.org/learn/get-started) page or just `curl https://sh.rustup.rs -sSf | sh` and follow the instruction.

### Redis
The Interledger.rs nodes currently use [Redis](https://redis.io/) to store their data (SQL database support coming soon!)

- Compile and install from the source code
    - [Download the source code here](https://redis.io/download)
- Install using package managers
    - Ubuntu: run `sudo apt-get install redis-server`
    - macOS: If you use Homebrew, run `brew install redis`

Make sure your Redis is empty. You could run `redis-cli flushall` to clear all the data.

## Instructions

<!--!
# import some functions from run-md-lib.sh
# this variable is set by run-md.sh
source $RUN_MD_LIB
init

printf "Stopping Interledger nodes...\n"

if [ "$USE_DOCKER" -eq 1 ]; then
    $CMD_DOCKER --version > /dev/null || error_and_exit "Uh oh! You need to install Docker before running this example"
    
    $CMD_DOCKER stop \
        interledger-rs-node_a \
        interledger-rs-node_b \
        redis-alice_node \
        redis-bob_node 2>/dev/null
    
    printf "\n\nRemoving existing Interledger containers\n"
    $CMD_DOCKER rm \
        interledger-rs-node_a \
        interledger-rs-node_b \
        redis-alice_node \
        redis-bob_node 2>/dev/null
else
    for port in 6379 6380; do
        if lsof -Pi :${port} -sTCP:LISTEN -t ; then
            redis-cli -p ${port} shutdown
        fi
    done
    
    if [ -f dump.rdb ] ; then
        rm -f dump.rdb
    fi
    
    for port in 8545 7770 8770; do
        if lsof -tPi :${port} ; then
            kill `lsof -tPi :${port}`
        fi
    done
fi

printf "\n"
-->

### 1. Build interledger.rs
First of all, let's build interledger.rs. (This may take a couple of minutes)

<!--!
if [ "$USE_DOCKER" -eq 1 ]; then
    NETWORK_ID=`$CMD_DOCKER network ls -f "name=interledger" --format="{{.ID}}"`
    if [ -z "${NETWORK_ID}" ]; then
        printf "Creating a docker network...\n"
        $CMD_DOCKER network create interledger
    fi
else
    printf "\nBuilding interledger.rs... (This may take a couple of minutes)\n\n"
-->
```bash
cargo build --all-features --bin interledger --bin ilp-cli
```
<!--!
fi
-->

### 2. Launch Redis

<!--!
printf "\n\nStarting Redis instances...\n\n"
if [ "$USE_DOCKER" -eq 1 ]; then
    $CMD_DOCKER run --name redis-alice_node -d -p 127.0.0.1:6379:6379 --network=interledger redis:5.0.5
    $CMD_DOCKER run --name redis-bob_node -d -p 127.0.0.1:6380:6379 --network=interledger redis:5.0.5
else
    redis-server --version > /dev/null || error_and_exit "Uh oh! You need to install redis-server before running this example"
-->

```bash
# Create the logs directory if it doesn't already exist
mkdir -p logs

# Start Redis
redis-server --port 6379 &> logs/redis-a-node.log &
redis-server --port 6380 &> logs/redis-b-node.log &
```
<!--!
sleep 1
-->

To remove all the data in Redis, you might additionally perform:

```bash
for port in `seq 6379 6380`; do
    redis-cli -p $port flushall
done
```
<!--!
fi
-->

When you want to watch logs, use the `tail` command. You can use the command like: `tail -f logs/redis-a-node.log`

### 3. Launch 2 Nodes

<!--!
printf "\n\nStarting Interledger nodes...\n"
if [ "$USE_DOCKER" -eq 1 ]; then
    $CMD_DOCKER run \
        -e ILP_ADDRESS=example.node_a \
        -e ILP_SECRET_SEED=8852500887504328225458511465394229327394647958135038836332350604 \
        -e ILP_ADMIN_AUTH_TOKEN=admin-example \
        -e ILP_REDIS_URL=redis://redis-alice_node:6379/ \
        -e ILP_HTTP_BIND_ADDRESS=0.0.0.0:7770 \
        -e ILP_SETTLEMENT_API_BIND_ADDRESS=0.0.0.0:7771 \
        -p 127.0.0.1:7770:7770 \
        -p 127.0.0.1:7771:7771 \
        --network=interledger \
        --name=interledger-rs-node_a \
        -td \
        interledgerrs/node node
    
    $CMD_DOCKER run \
        -e ILP_ADDRESS=example.node_b \
        -e ILP_SECRET_SEED=1604966725982139900555208458637022875563691455429373719368053354 \
        -e ILP_ADMIN_AUTH_TOKEN=admin-example \
        -e ILP_REDIS_URL=redis://redis-bob_node:6379/ \
        -e ILP_HTTP_BIND_ADDRESS=0.0.0.0:7770 \
        -e ILP_SETTLEMENT_API_BIND_ADDRESS=0.0.0.0:7771 \
        -p 127.0.0.1:8770:7770 \
        -p 127.0.0.1:8771:7771 \
        --network=interledger \
        --name=interledger-rs-node_b \
        -td \
        interledgerrs/node node
else
-->

```bash
# Turn on debug logging for all of the interledger.rs components
export RUST_LOG=interledger=debug

# Start both nodes.
# Note that the configuration options can be passed as environment variables
# or saved to a YAML, JSON or TOML file and passed to the node as a positional argument.
# You can also pass it from STDIN.
ILP_ADDRESS=example.node_a \
ILP_SECRET_SEED=8852500887504328225458511465394229327394647958135038836332350604 \
ILP_ADMIN_AUTH_TOKEN=admin-example \
ILP_REDIS_URL=redis://127.0.0.1:6379/ \
ILP_HTTP_BIND_ADDRESS=127.0.0.1:7770 \
ILP_SETTLEMENT_API_BIND_ADDRESS=127.0.0.1:7771 \
cargo run --all-features --bin interledger -- node &> logs/node_a.log &

ILP_ADDRESS=example.node_b \
ILP_SECRET_SEED=1604966725982139900555208458637022875563691455429373719368053354 \
ILP_ADMIN_AUTH_TOKEN=admin-example \
ILP_REDIS_URL=redis://127.0.0.1:6380/ \
ILP_HTTP_BIND_ADDRESS=127.0.0.1:8770 \
ILP_SETTLEMENT_API_BIND_ADDRESS=127.0.0.1:8771 \
cargo run --all-features --bin interledger -- node &> logs/node_b.log &
```

<!--!
fi

printf "\nWaiting for Interledger.rs nodes to start up"

wait_to_serve "http://localhost:7770" 10 || error_and_exit "\nFailed to spin up nodes. Check out your configuration and log files."
wait_to_serve "http://localhost:8770" 10 || error_and_exit "\nFailed to spin up nodes. Check out your configuration and log files."

printf " done\nThe Interledger.rs nodes are up and running!\n\n"
-->

Now the Interledger.rs nodes are up and running!  
You can also watch the logs with: `tail -f logs/node_a.log` or `tail -f logs/node_b.log`.

### 4. Configure the Nodes

Let's create accounts on both nodes. The following script sets up accounts for two users, Alice and Bob. It also creates accounts that represent the connection between Nodes A and B.

See the [HTTP API docs](../../docs/api.md) for the full list of fields that can be set on an account.

<!--!
printf "\nCreating accounts...\n\n"

if [ "$USE_DOCKER" -eq 1 ]; then
    printf "Alice's account:\n"
    curl \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer admin-example" \
        -d '{
        "ilp_address": "example.node_a.alice",
        "username" : "alice",
        "asset_code": "ABC",
        "asset_scale": 9,
        "max_packet_amount": 100,
        "ilp_over_http_incoming_token": "alice-password"}' \
        http://localhost:7770/accounts
    
    printf "\nNode B's account on Node A:\n"
    curl \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer admin-example" \
        -d '{
        "ilp_address": "example.node_b",
        "username" : "node_b",
        "asset_code": "ABC",
        "asset_scale": 9,
        "max_packet_amount": 100,
        "ilp_over_http_incoming_token": "node_b-password",
        "ilp_over_http_outgoing_token": "node_a:node_a-password",
        "ilp_over_http_url": "http://interledger-rs-node_b:7770/ilp",
        "min_balance": -100000,
        "routing_relation": "Peer"}' \
        http://localhost:7770/accounts
    
    # Insert accounts on Node B
    # One account represents Bob and the other represents Node A's account with Node B
    
    printf "\nBob's Account:\n"
    curl \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer admin-example" \
        -d '{
        "ilp_address": "example.node_b.bob",
        "username" : "bob",
        "asset_code": "ABC",
        "asset_scale": 9,
        "max_packet_amount": 100,
        "ilp_over_http_incoming_token": "bob"}' \
        http://localhost:8770/accounts
    
    printf "\nNode A's account on Node B:\n"
    curl \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer admin-example" \
        -d '{
        "ilp_address": "example.node_a",
        "username" : "node_a",
        "asset_code": "ABC",
        "asset_scale": 9,
        "max_packet_amount": 100,
        "ilp_over_http_incoming_token": "node_a-password",
        "ilp_over_http_outgoing_token": "node_b:node_b-password",
        "ilp_over_http_url": "http://interledger-rs-node_a:7770/ilp",
        "min_balance": -100000,
        "routing_relation": "Peer"}' \
        http://localhost:8770/accounts
else
-->

```bash
# Insert accounts on Node A
# One account represents Alice and the other represents Node B's account with Node A
export ILP_CLI_AUTH=admin-example

printf "Creating Alice's account on Node A...\n"
cargo run --quiet --bin ilp-cli -- \
    add-account alice ABC 9 \
    --ilp-over-http-incoming-token alice-password

printf "Creating Node B's account on Node A...\n"
cargo run --quiet --bin ilp-cli -- \
    add-account node_b ABC 9 \
    --ilp-address example.node_b \
    --ilp-over-http-outgoing-token node_a:node_a-password \
    --ilp-over-http-url 'http://localhost:8770/ilp'

# Insert accounts on Node B
# One account represents Bob and the other represents Node A's account with Node B

printf "Creating Bob's account on Node B...\n"
cargo run --quiet --bin ilp-cli -- \
    --node localhost:8770 \
    add-account bob ABC 9

printf "Creating Node A's account on Node B...\n"
cargo run --quiet --bin ilp-cli -- \
    --node localhost:8770 \
    add-account node_a ABC 9 \
    --ilp-over-http-incoming-token node_a-password
```

<!--!
fi
-->

### 5. Sending a Payment

<!--!
# check balances before payment
printf "\n\nChecking balances prior to payment...\n"
printf "\nAlice's balance: "
cargo run --quiet --bin ilp-cli -- \
    --print-response \
    get-balance alice

printf "Node B's balance on Node A: "
cargo run --quiet --bin ilp-cli -- \
    --print-response \
    get-balance node_b

printf "Node A's balance on Node B: "
cargo run --quiet --bin ilp-cli -- \
    --node localhost:8770 \
    --print-response \
    get-balance node_a

printf "Bob's balance: "
cargo run --quiet --bin ilp-cli -- \
    --node localhost:8770 \
    --print-response \
    get-balance bob

printf "\n\n"
-->

The following command sends a payment from Alice to Bob that is routed from Node A to Node B.

<!--!
printf "Sending payment of 500 from Alice (on Node A) to Bob (on Node B)...\n\n"

if [ "$USE_DOCKER" -eq 1 ]; then
    curl \
        -H "Authorization: Bearer alice:alice-password" \
        -H "Content-Type: application/json" \
        -d "{\"receiver\":\"http://interledger-rs-node_b:7770/accounts/bob/spsp\",\"source_amount\":500}" \
        http://localhost:7770/accounts/alice/payments
else
-->

```bash
# Sending payment of 500 from Alice (on Node A) to Bob (on Node B)
cargo run --quiet --bin ilp-cli -- \
    --auth alice-password \
    --print-response \
    post-payment alice 500 'http://localhost:8770/accounts/bob/spsp'
```

<!--!
fi
printf "\n\n"
-->

### 6. Check Balances

You can run the following script to print each of the accounts' balances (try doing this before and after sending a payment).

<!--! printf "Checking balances after payment...\n" -->

```bash
printf "\nAlice's balance: "
cargo run --quiet --bin ilp-cli -- \
    --print-response \
    get-balance alice

printf "Node B's balance on Node A: "
cargo run --quiet --bin ilp-cli -- \
    --print-response \
    get-balance node_b

printf "Node A's balance on Node B: "
cargo run --quiet --bin ilp-cli -- \
    --node localhost:8770 \
    --print-response \
    get-balance node_a

printf "Bob's balance: "
cargo run --quiet --bin ilp-cli -- \
    --node localhost:8770 \
    --print-response \
    get-balance bob
```

<!--!
printf "\n\n"
-->

### 7. Kill All the Services
Finally, you can stop all the services as follows:

<!--!
run_hook_before_kill
if [ $TEST_MODE -ne 1 ]; then
    prompt_yn "Do you want to kill the services? [Y/n] " "y"
fi
printf "\n"
if [ "$PROMPT_ANSWER" = "y" ] || [ $TEST_MODE -eq 1 ] ; then
    if [ "$USE_DOCKER" -ne 1 ]; then
        exec 2> /dev/null
-->
```bash
for port in 6379 6380; do
    if lsof -Pi :${port} -sTCP:LISTEN -t >/dev/null ; then
        redis-cli -p ${port} shutdown
    fi
done

if [ -f dump.rdb ] ; then
    rm -f dump.rdb
fi

for port in 8545 7770 8770; do
    if lsof -tPi :${port} >/dev/null ; then
        kill `lsof -tPi :${port}` > /dev/null
    fi
done
```
<!--!
    else
-->

If you are using Docker, try the following.

<!--!
        $CMD_DOCKER stop \
            interledger-rs-node_a \
            interledger-rs-node_b \
            redis-alice_node \
            redis-bob_node
    fi
fi
printf "\n"
-->
```bash #
# Depending on your OS, you might not need to prefix with `sudo` necessarily.
sudo docker stop \
    interledger-rs-node_a \
    interledger-rs-node_b \
    redis-alice_node \
    redis-bob_node
```

## Troubleshooting

```
Uh oh! You need to install redis-server before running this example
```

You need to install Redis to run `redis-server` command. See [Prerequisites](#prerequisites) section.

```
curl: (7) Failed to connect to localhost port 7770: Connection refused
```

Your interledger.rs node is not running. The reason may be:

1. You tried to insert the accounts before the nodes had time to spin up. Wait a second or so, and try again.
1. You have already running interledger.rs nodes on port `7770`. Stop the nodes and retry.
1. You have some other process running on port `7770`. Stop the process and retry.

To stop the process running on port `7770`, try `` kill `lsof -i:7770 -t` ``. Since this example launches 2 nodes, the port may be other than `7770`. Adjust the port number according to the situation.

```
# When running with Docker
Error starting userland proxy: listen tcp 0.0.0.0:6379: bind: address already in use.
```

You might have run another example. Stop them first and try again. How to stop the services is written in each example page.

## Conclusion

That's it for this example! You've learned how to set up Interledger.rs nodes, connect them together, and how to send a payment from one to the other.

Check out the [other examples](../README.md) for more complex demos that show other features of Interledger, including settlement, multi-hop routing, and cross-currency payments.

<!--!
# For integration tests
function hook_before_kill() {
    if [ $TEST_MODE -eq 1 ]; then
        test_equals_or_exit '{"balance":"-500"}' test_http_response_body -H "Authorization: Bearer admin-example" http://localhost:7770/accounts/alice/balance
        test_equals_or_exit '{"balance":"500"}' test_http_response_body -H "Authorization: Bearer admin-example" http://localhost:7770/accounts/node_b/balance
        test_equals_or_exit '{"balance":"-500"}' test_http_response_body -H "Authorization: Bearer admin-example" http://localhost:8770/accounts/node_a/balance
        test_equals_or_exit '{"balance":"500"}' test_http_response_body -H "Authorization: Bearer admin-example" http://localhost:8770/accounts/bob/balance
    fi
}
-->
