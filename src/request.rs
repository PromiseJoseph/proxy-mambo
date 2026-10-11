use std::io::Result;
use std::io::{Error, ErrorKind};
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

pub fn update_request_headers(headers: &[u8], upstream_addr: &str) -> Result<Vec<u8>> {
    if !headers.ends_with(b"\r\n\r\n") {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Incomplete HTTP headers",
        ));
    }

    // Prevent the configured Host value from injecting another header.
    if upstream_addr.contains('\r') || upstream_addr.contains('\n') {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Invalid upstream Host value",
        ));
    }

    // Remove the final header separator temporarily.
    let header_content = &headers[..headers.len() - 4];

    let mut updated = Vec::with_capacity(headers.len() + upstream_addr.len());
    let mut found_host = false;

    for (index, raw_line) in header_content.split(|byte| *byte == b'\n').enumerate() {
        // Remove the trailing CR so we can inspect the header line.
        let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);

        let is_host = index != 0
            && line
                .get(..5)
                .map_or(false, |name| name.eq_ignore_ascii_case(b"host:"));

        if is_host {
            if found_host {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Request contains multiple Host headers",
                ));
            }

            found_host = true;

            updated.extend_from_slice(b"Host: ");
            updated.extend_from_slice(upstream_addr.as_bytes());
            updated.extend_from_slice(b"\r\n");
        } else {
            updated.extend_from_slice(line);
            updated.extend_from_slice(b"\r\n");
        }
    }

    // If the incoming request had no Host header, add one.
    if !found_host {
        updated.extend_from_slice(b"Host: ");
        updated.extend_from_slice(upstream_addr.as_bytes());
        updated.extend_from_slice(b"\r\n");
    }

    // Restore the blank line separating headers from the body.
    updated.extend_from_slice(b"\r\n");

    Ok(updated)
}
