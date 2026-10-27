<!-- markdownlint-disable -->
<div align="center">
  <h1>ProxyMambo<br></h1>
  <p>
    <strong>
      A reverse proxy and load balancer built from scratch in Rust . It explores TCP networking, HTTP request forwarding, path-based
      routing, and load balancing algorithms.
    </strong>
  </p>
  <p>
    <a href="#features">Features</a> ·
    <a href="#getting-started">Getting Started</a> ·
    <a href="#example">Example</a> ·
    <a href="#architecture">Architecture</a>
  </p>
</div>

---------------------------------------------------------------------

## Overview

ProxyMambo is a learning project focused on understanding how reverse
proxies and load balancers work internally. 

The project is built with Rust, using Tokio for the asynchronous ntworking and connection handlings.. Accepts client TCP connections, reads the initial HTTP request headers, routes requests to configured upstream pools based on the request path and configured load balancing algorithms.. modifies selected headers, and relay traffic between clients and upstream servers.

This project has being developed incrementally from the fundamentals.. with no reliance on an existing reverse proxy framework.

## Features

-   Asynchronous TCP server using Tokio
-   Concurrent client connection handling with Tokio tasks
-   Path-based routing to configured upstream pools
-   Round Robin load balancing
-   Weighted Round Robin load balancing
-   HTTP request header modification, including `Host`,
    `X-Forwarded-For`, and `X-Forwarded-Host`
-   Configurable maximum HTTP header size
-   Bidirectional stream relay between clients and upstream servers
-   Shared load-balancing state protected with atomic operations and an asynchronous mutex

## Getting Started

### Prerequisites

-   Rust and Cargo
-   An API upstream server, defaulting to `127.0.0.1:8080`
-   Three guest upstream servers on `127.0.0.1:8001`, `127.0.0.1:8002`, and `127.0.0.1:8003`

The upstream addresses are currently configured in `main.rs` for local
testing. You can change them to match your setup.

### Run ProxyMambo

Clone the repository and enter its directory:

``` bash
git clone https://github.com/PromiseJoseph/ProxyMambo.git
cd ProxyMambo
```

Run the project:

``` bash
cargo run
```

By default.. ProxyMambo uses the proxy listening address defined in
`Config`.

You can provide a custom proxy address and API upstream address as
positional arguments:

``` bash
cargo run -- 127.0.0.1:7777 127.0.0.1:8080
```

The first argument is for the listening address,  while the second is for the API upstream address.

If the `PORT` environment variable is set, ProxyMambo listens on
`0.0.0.0:<PORT>`, for deployment environments that provide a port
through an environment variable.

## Example

With ProxyMambo listening on `127.0.0.1:7777`, send a request to the API route:

``` bash
curl -i http://127.0.0.1:7777/api
```

Send a request to the guest route:

``` bash
curl -i http://127.0.0.1:7777/
```

To observe weighted upstream selection across separate client
connections:

``` bash
for i in $(seq 1 14); do
  curl -s -o /dev/null -w "%{http_code}\n" \
    -H "Connection: close" \
    http://127.0.0.1:7777/
done
```

The `Connection: close` header asks the client to close each connection after the request.. This helps demonstrate upstream selection because the current proxy selects an upstream once per client TCP connection.

## Architecture

``` text
Client
  |
  v
ProxyMambo
  |
  v
HTTP Header Parsing
  |
  v
Path-based Router
  |
  +-------------------+
  |                   |
  v                   v
/api                  /
  |                   |
  v                   v
API Pool              Guest Pool
  |                   |
  v                   v
Round Robin           Smooth Weighted Round Robin
  |                   |
  v                   v
API Upstream          Guest Upstreams
  |                   |
  +---------+---------+
            |
            v
     Bidirectional Relay
            |
            v
          Client
```

## Routing

ProxyMambo currently defines two routes for testing and demonstration of path-based routing and load balancing:

| Request path | Upstream pool | Algorithm |
| --- | --- | --- |
| `/api` | `api` | Round Robin |
| `/api/*` | `api` | Round Robin |
| `/` | `guest` | Smooth Weighted Round Robin |

Routes are currently matched by exact path and wildcard patterns.. For example, `/api` matches only the exact `/api` path, while `/api/*` matches /api & nested paths under `/api` such as `/api/users`.

## Load Balancing

### Round Robin

Round Robin distributes requests across upstream servers in sequence, cycling back to the first server after reaching the last. An atomic counter tracks the next selection, allowing concurrent tasks to update the counter safely.

### Weighted Round Robin

Smooth Weighted Round Robin distributes requests according to the configured weights of upstream servers.. adjusts each server's current weight during every selection to distribute traffic proportionally while keeping the selection sequence relatively smooth.

| Upstream | Weight | Expected share over many selections |
| --- | ---: | ---: |
| `guest-1` | 5 | 71.4% |
| `guest-2` | 1 | 14.3% |
| `guest-3` | 1 | 14.3% |

These percentages describe the expected distribution over many
selections when all three upstreams are available. The algorithm
maintains shared current weights using `tokio::sync::Mutex` so
concurrent tasks can update the selection state safely.

## Configuration

The current configuration is defined programmatically in `main.rs` and
includes:

-   Proxy listening address
-   API upstream address
-   Guest upstream addresses
-   Routes and associated pool IDs
-   Load-balancing algorithm per pool
-   Weights for the guest upstreams
-   Maximum HTTP header size, default is `16384` bytes

The upstream addresses and weights are currently set up for local testing.

## Project Structure

``` text
src/
    algorithm.rs
    config.rs
    proxy.rs
    request.rs
    stream.rs
    types.rs
```

## Current Development

ProxyMambo is actively being developed with an upcoming focus on
reliability and upstream failure handling,timeouts handling, logging &observability,shutdowns and explicit HTTP error responses.

Planned improvements include:

-   Upstream connection timeouts
-   Request header read timeouts
-   Explicit HTTP error responses, including `502 Bad Gateway` and
    `504 Gateway Timeout`
-   Handling unavailable upstream servers and empty pools
-   Tests for routing and load balancing
-   Least Connections load balancing
-   Logging and observability
-   Graceful shutdowns

## Motivation

I built ProxyMambo to deepen my understanding of TCP networking, HTTP
request forwarding, asynchronous programming, shared state, and load
balancing in Rust. It is part of my ongoing journey into systems
programming, backend infrastructure, and networking.

## License
-
