use crate::types::{Config, LoadBalancingAlgorithm, Route, Upstream, UpstreamPool};
use std::sync::{Arc, atomic::AtomicUsize};

impl Config {
    pub fn new() -> Self {
        Self {
            proxy_addr: "127.0.0.1:8080".to_string(),
            upstream_pools: vec![UpstreamPool {
                id: "default".to_string(),
                algorithm: LoadBalancingAlgorithm::RoundRobin {
                    current_index: Arc::new(AtomicUsize::new(0)),
                },
                upstreams: vec![Upstream {
                    id: "default-main".to_string(),
                    addr: "127.0.0.1:8000".to_string(),
                }],
            }],
            max_header_size: 16384, // Default maximum header size set to 16KiB
            routes: vec![],
        }
    }

    /* ===Upstream management methods=== */

    pub fn add_upstream(&mut self, upstream_pool_id: &str, upstream: Upstream) {
        if let Some(pool) = self
            .upstream_pools
            .iter_mut()
            .find(|p| p.id == upstream_pool_id)
        {
            pool.upstreams.push(upstream);
        }
    }

    pub fn add_upstream_pool(
        &mut self,
        id: &str,
        upstreams: Vec<Upstream>,
        lba: LoadBalancingAlgorithm,
    ) {
        self.upstream_pools.push(UpstreamPool {
            id: id.to_string(),
            algorithm: lba,
            upstreams,
        });
    }

    pub fn new_upstream_pools(&mut self, upstream_pools: Vec<UpstreamPool>) {
        self.upstream_pools = upstream_pools;
    }

    pub fn get_upstream(&self, pool_id: &str, upstream_id: &str) -> Option<&Upstream> {
        if let Some(pool) = self.upstream_pools.iter().find(|p| p.id == pool_id) {
            pool.upstreams.iter().find(|u| u.id == upstream_id)
        } else {
            None
        }
    }
    pub fn get_upstream_pool(&self, pool_id: &str) -> Option<&UpstreamPool> {
        self.upstream_pools.iter().find(|p| p.id == pool_id)
    }

    pub fn get_upstream_pools(&self) -> &[UpstreamPool] {
        &self.upstream_pools
    }

    pub fn check_upstream_exists(&self, pool_id: &str, upstream_id: &str) -> bool {
        self.upstream_pools
            .iter()
            .any(|p| p.id == pool_id && p.upstreams.iter().any(|u| u.id == upstream_id))
    }
    pub fn check_upstream_pool_exists(&self, id: &str) -> bool {
        self.upstream_pools.iter().any(|p| p.id == id)
    }

    pub fn get_pool_algorithm(&self, pool_id: &str) -> Option<&LoadBalancingAlgorithm> {
        self.upstream_pools
            .iter()
            .find(|p| p.id == pool_id)
            .map(|p| &p.algorithm)
    }

    /* ===Proxy address and header size management methods=== */
    pub fn set_proxy_addr(&mut self, addr: &str) {
        self.proxy_addr = addr.to_string();
    }

    pub fn set_max_header_size(&mut self, size: usize) {
        self.max_header_size = size;
    }

    pub fn get_proxy_addr(&self) -> &str {
        &self.proxy_addr
    }

    pub fn get_max_header_size(&self) -> usize {
        self.max_header_size
    }

    /* ===Route management methods=== */
    pub fn set_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn set_routes(&mut self, routes: Vec<Route>) {
        self.routes = routes;
    }

    pub fn get_routes(&self) -> &[Route] {
        &self.routes
    }

    pub fn get_route(&self, path: &str) -> Option<&Route> {
        self.routes.iter().find(|r| r.path == path)
    }

    pub fn check_route_exists(&self, path: &str) -> bool {
        self.routes.iter().any(|r| r.path == path)
    }
}
