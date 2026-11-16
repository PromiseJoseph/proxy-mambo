use std::sync::{Arc, atomic::AtomicUsize};
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Client {
    pub peer_addr: SocketAddr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub path: String,
    pub pool_id: String,
}

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
pub(crate) struct Config {
    pub(crate) proxy_addr: String,
    pub(crate) upstream_pools: Vec<UpstreamPool>,
    pub(crate) max_header_size: usize,
    pub(crate) routes: Vec<Route>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorResponse {
    BadRequest,
    NotFound,
    RequestHeaderFieldsTooLarge,
    InternalServerError,
    BadGateway,
    ServiceUnavailable,
    Forbidden,
}
