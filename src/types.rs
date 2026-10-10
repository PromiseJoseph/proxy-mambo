use std::future::Future;
use std::pin::Pin;
use std::{collections::HashMap, net::SocketAddr};

#[derive(Debug, Clone, PartialEq)]
pub struct RequestLines {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub request_lines: RequestLines,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub peer_addr: SocketAddr,
}

pub(crate) type BoxFuture = Pin<Box<dyn Future<Output = HttpResponse> + Send>>;

pub(crate) type Handler = Box<dyn Fn(HttpRequest) -> BoxFuture + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
}

pub(crate) struct Route {
    pub path: String,
    pub method: HttpMethod,
    pub handler: Handler,
}
pub struct Router {
    pub(crate) routes: Vec<Route>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

#[derive(Debug, PartialEq)]
pub(crate) enum ParamType {
    String,
    U16,
}
