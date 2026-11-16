use crate::types::ErrorResponse;

#[derive(Debug, Clone)]
pub enum ProxyError {
    BadRequest(Option<String>),
    NotFound(Option<String>),
    RequestHeaderFieldsTooLarge(Option<String>),
    UpstreamUnavailable(Option<String>),
    BadGateway(Option<String>),
    InternalServerError(Option<String>),
    Forbidden(Option<String>),
}

impl From<&ProxyError> for ErrorResponse {
    fn from(error: &ProxyError) -> Self {
        match error {
            ProxyError::BadRequest(_) => Self::BadRequest,
            ProxyError::NotFound(_) => Self::NotFound,
            ProxyError::RequestHeaderFieldsTooLarge(_) => Self::RequestHeaderFieldsTooLarge,
            ProxyError::UpstreamUnavailable(_) => Self::ServiceUnavailable,
            ProxyError::BadGateway(_) => Self::BadGateway,
            ProxyError::InternalServerError(_) => Self::InternalServerError,
            ProxyError::Forbidden(_) => Self::Forbidden,
        }
    }
}
