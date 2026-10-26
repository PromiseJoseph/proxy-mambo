use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, atomic::AtomicUsize};
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;

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

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub pool_id: String,
}

pub struct Router {
    pub(crate) routes: Vec<Route>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

#[derive(Debug, Clone)]
pub struct Upstream {
    pub id: String,
    pub addr: String,
}

#[derive(Debug)]
pub enum LoadBalancingAlgorithm {
    RoundRobin {
        current_index: Arc<AtomicUsize>,
    },
    WeightedRoundRobin {
        weights: HashMap<String, usize>,
        current_weights: Mutex<HashMap<String, isize>>,
    },
}

#[derive(Debug)]
pub struct UpstreamPool {
    pub id: String,
    pub upstreams: Vec<Upstream>,
    pub algorithm: LoadBalancingAlgorithm,
}

#[derive(Debug)]
pub struct Config {
    pub(crate) proxy_addr: String,
    pub(crate) upstream_pools: Vec<UpstreamPool>,
    pub(crate) max_header_size: usize,
    pub(crate) routes: Vec<Route>,
}
