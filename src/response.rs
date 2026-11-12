use crate::types::ErrorResponse;
use tokio::io::{AsyncWrite, AsyncWriteExt};

/*
    Represents different types of error responses that can be sent to the client.
*/
impl ErrorResponse {
    pub fn status(&self) -> u16 {
        match self {
            Self::BadRequest => 400,
            Self::NotFound => 404,
            Self::RequestHeaderFieldsTooLarge => 431,
            Self::InternalServerError => 500,
            Self::BadGateway => 502,
            Self::ServiceUnavailable => 503,
            Self::Forbidden => 403,
        }
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::BadRequest => "Bad Request",
            Self::NotFound => "Not Found",
            Self::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            Self::InternalServerError => "Internal Server Error",
            Self::BadGateway => "Bad Gateway",
            Self::ServiceUnavailable => "Service Unavailable",
            Self::Forbidden => "Forbidden",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::BadRequest => "Invalid HTTP request",
            Self::NotFound => "Route not found",
            Self::RequestHeaderFieldsTooLarge => "HTTP headers are too large",
            Self::InternalServerError => "An internal server error occurred",
            Self::BadGateway => "Failed to connect to upstream",
            Self::ServiceUnavailable => "No upstream server available",
            Self::Forbidden => "Access denied",
        }
    }
}

pub async fn send_error_response<W>(stream: &mut W, error: ErrorResponse) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let body = format!("{}\n", error.message());

    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        error.status(),
        error.reason(),
        body.len(),
        body
    );

    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await?;

    Ok(())
}
