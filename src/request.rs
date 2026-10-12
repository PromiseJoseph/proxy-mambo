use crate::types::Client;
use std::io::{Error, ErrorKind, Result};
/**
 *  Parsea request string and return a HttpRequest struct.
 */
pub fn update_request(request: &str) -> String {
    let request_header: Vec<&str> = request.split("\r\n\r\n").collect();

    let updated_request_header = request_header[0].to_string() + "\r\n\r\n"; //test'
    let updated_request = updated_request_header + &request_header[1];

    return updated_request.to_string();
}

pub fn update_response(response: &str) -> String {
    let response_header: Vec<&str> = response.split("\r\n\r\n").collect();

    let updated_response_header = response_header[0].to_string() + "\r\n\r\n"; //test'
    let updated_response = updated_response_header + &response_header[1];

    return updated_response.to_string();
}

pub fn update_request_headers(
    headers: &[u8],
    upstream_host: &str,
    client: &Client,
) -> Result<Vec<u8>> {
    if !headers.ends_with(b"\r\n\r\n") {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Incomplete HTTP headers",
        ));
    }

    // Prevent the configured Host value from injecting another header.
    if upstream_host.contains('\r') || upstream_host.contains('\n') {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid upstream Host value",
        ));
    }

    // Remove the final header separator temporarily.
    let header_content = &headers[..headers.len() - 4];

    let mut updated = Vec::with_capacity(headers.len() + 128); //updated to be more efficient.. preallocate the vector with enough capacity to hold the original headers and the new header items.
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

    Ok(updated)
}
