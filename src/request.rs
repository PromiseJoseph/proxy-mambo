use crate::types::{Client, Config, Route, UpstreamPool};
use std::{
    io::{Error, ErrorKind, Result},
    println,
};
/**
 *  Parse request string and return a HttpRequest struct.
 */

pub fn update_request_headers(
    header: &[u8],
    client: &Client,
    config: &Config,
) -> Result<(Vec<u8>, String)> {
    if !header.ends_with(b"\r\n\r\n") {
        return Err(Error::new(ErrorKind::InvalidData, "Incomplete HTTP header")); // to be addressed as a http response with status code 400 Bad Request..
    }

    // Remove the final header separator temporarily.
    let header_content = &header[..header.len() - 4];

    let (method, path, version) = parse_request_line(header_content)?;
    println!(
        "Parsed request line: method={}, path={}, version={}",
        method, path, version
    );
    let upstream = find_upstream_for_path(path, config);

    if upstream.is_none() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "No upstream found for the requested path",
        ));
    } //to be addressed as a http response with status code 404 Not Found..

    let upstream_hosts = upstream.unwrap().upstreams; //to be addressed as a http response with status code 404 Not Found..

    // Prevent the configured Host value from injecting another header.
    for host in upstream_hosts {
        if host.addr.contains('\r') || host.addr.contains('\n') {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid upstream Host value",
            ));
        }
    }
    //round robbin here...

    let mut updated = Vec::with_capacity(header.len() + 128); //updated to be more efficient.. preallocate the vector with enough capacity to hold the original header and the new header items.
    let mut incoming_host: Option<Vec<u8>> = None;

    for (index, raw_line) in header_content.split(|byte| *byte == b'\n').enumerate() {
        // Remove the trailing CR so we can inspect the header line.
        let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);

        let colon = line.iter().position(|b| *b == b':');
        let name = colon.map(|pos| &line[..pos]).unwrap_or(&[]);

        let is_host = index > 0 && name.eq_ignore_ascii_case(b"Host");

        let is_forwarded_header = index > 0
            && (name.eq_ignore_ascii_case(b"x-forwarded-for")
                || name.eq_ignore_ascii_case(b"x-forwarded-host"));

        if is_host {
            if incoming_host.is_some() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Request contains multiple Host headers",
                ));
            }

            // Extract the value of the Host header.
            let value = &line[colon.unwrap() + 1..];
            let value = value.strip_prefix(b" ").unwrap_or(value);

            incoming_host = Some(value.to_vec());

            updated.extend_from_slice(b"Host: ");
            updated.extend_from_slice(upstream_host.as_bytes());
            updated.extend_from_slice(b"\r\n");
        } else if is_forwarded_header {
            // Replace any client-supplied forwarding headers.
            continue;
        } else {
            updated.extend_from_slice(line);
            updated.extend_from_slice(b"\r\n");
        }
    }

    //Add Host if the request did not contain one.
    if incoming_host.is_none() {
        updated.extend_from_slice(b"Host: ");
        updated.extend_from_slice(upstream_host.as_bytes());
        updated.extend_from_slice(b"\r\n");
    }
    // Preserve the host the client originally requested.. to be changed to a validated or fixed host value.
    if let Some(host) = incoming_host {
        updated.extend_from_slice(b"X-Forwarded-Host: ");
        updated.extend_from_slice(&host);
        updated.extend_from_slice(b"\r\n");
    }

    // Record the remote client's IP.
    updated.extend_from_slice(b"X-Forwarded-For: ");
    updated.extend_from_slice(&client.peer_addr.ip().to_string().as_bytes());
    updated.extend_from_slice(b"\r\n");

    // Restore the blank line separating headers from the body.
    updated.extend_from_slice(b"\r\n");

    Ok((updated, upstream_host))
}

// extract the method, path, and version from the request line.
fn parse_request_line(raw_header: &[u8]) -> Result<(&str, &str, &str)> {
    let request_header = std::str::from_utf8(raw_header)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "HTTP header is not valid UTF-8"))?; // to be addressed as a http response with status code 400 Bad Request..

    let request_line = request_header
        .lines()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing HTTP request line"))?; // to be addressed as a http response with status code 400 Bad Request..

    let mut parts = request_line.split_whitespace();

    let method = parts
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing HTTP method"))?; // to be addressed as a http response with status code 400 Bad Request..
    let path = parts
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing request path"))?; // to be addressed as a http response with status code 400 Bad Request..
    let version = parts
        .next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing HTTP version"))?; // to be addressed as a http response with status code 400 Bad Request..
    return Ok((method, path, version));
}

// Find the upstream route for a given request path.
fn find_upstream_for_path<'a>(path: &str, config: &'a Config) -> Option<&'a UpstreamPool> {
    let routes: &[Route] = config.get_routes();
    let trimmed_path = path.trim_start_matches('/'); // Remove leading slashes for matching
    let upstream = config.get_upstream_pools(); // Get the default upstream from the config
    println!(
        "Finding upstream for path: {}, Routes: {:?}, Upstream: {:?}",
        trimmed_path, routes, upstream
    );
    let route = routes.iter().find(|route| route.path == path); // Remove leading slashes for matching

    if let Some(route) = route {
        config.get_upstream_pool(&route.pool_id) // Use the pool_id to find the upstream
    } else {
        None
    }
}
