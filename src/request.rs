use crate::proxy;

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
