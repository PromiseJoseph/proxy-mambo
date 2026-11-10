use crate::types::ErrorResponse;
pub enum ProxyError {
    BadRequest,
    NotFound,
    RequestHeaderFieldsTooLarge,
    UpstreamUnavailable,
    BadGateway,
    InternalServerError,
}

impl From<ProxyError> for ErrorResponse {
    fn from(error: ProxyError) -> Self {
        match error {
            ProxyError::BadRequest => Self::BadRequest,
            ProxyError::NotFound => Self::NotFound,
            ProxyError::RequestHeaderFieldsTooLarge => Self::RequestHeaderFieldsTooLarge,
            ProxyError::UpstreamUnavailable => Self::ServiceUnavailable,
            ProxyError::BadGateway => Self::BadGateway,
            ProxyError::InternalServerError => Self::InternalServerError,
        }
    }
}
