use crate::types::{Config, LoadBalancingAlgorithm, Route, Upstream, UpstreamPool};

use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    fs, println,
    sync::{Arc, atomic::AtomicUsize},
};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub server: ServerSettings,
    pub logging: LoggingSettings,
    pub upstream_pools: Vec<UpstreamPoolSettings>,
    pub routes: Vec<RouteSettings>,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub address: String,
    pub max_header_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoggingSettings {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct UpstreamPoolSettings {
    pub id: String,
    pub algorithm: AlgorithmSettings,
    pub upstreams: Vec<UpstreamSettings>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlgorithmSettings {
    RoundRobin,
    WeightedRoundRobin,
}

#[derive(Debug, Deserialize)]
pub struct UpstreamSettings {
    pub id: String,
    pub address: String,
    pub weight: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RouteSettings {
    pub path: String,
    pub pool_id: String,
}

// ==========================================
// TOML loading and runtime conversion
// ==========================================

impl AppSettings {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let contents = fs::read_to_string(path)?;
        let settings: Self = toml::from_str(&contents)?;
        println!("Loaded configuration from {}: {:?}", path, settings); //temporary debug print

        Ok(settings)
    }

    pub fn into_runtime_config(self) -> Result<Config, String> {
        let mut config = Config::new();

        config.set_proxy_addr(&self.server.address);
        config.set_max_header_size(self.server.max_header_size);

        let mut upstream_pools = Vec::new();

        for pool in self.upstream_pools {
            if pool.upstreams.is_empty() {
                return Err(format!(
                    "Upstream pool '{}' has no upstream servers",
                    pool.id
                ));
            }

            let mut upstreams = Vec::new();
            let mut weights = HashMap::new();
            let mut current_weights = HashMap::new();

            for upstream in pool.upstreams {
                if let Some(weight) = upstream.weight {
                    if weight == 0 {
                        return Err(format!("Upstream '{}' has a weight of zero", upstream.id));
                    }

                    weights.insert(upstream.id.clone(), weight);
                }

                current_weights.insert(upstream.id.clone(), 0);

                upstreams.push(Upstream {
                    id: upstream.id,
                    addr: upstream.address,
                });
            }

            let algorithm = match pool.algorithm {
                AlgorithmSettings::RoundRobin => LoadBalancingAlgorithm::RoundRobin {
                    current_index: Arc::new(AtomicUsize::new(0)),
                },

                AlgorithmSettings::WeightedRoundRobin => {
                    if weights.len() != upstreams.len() {
                        return Err(format!(
                            "Every upstream in pool '{}' must have a weight",
                            pool.id
                        ));
                    }

                    LoadBalancingAlgorithm::WeightedRoundRobin {
                        weights,
                        current_weights: Mutex::new(current_weights),
                    }
                }
            };

            upstream_pools.push(UpstreamPool {
                id: pool.id,
                algorithm,
                upstreams,
            });
        }

        // Ensure every route references an existing upstream pool.
        let routes: Vec<Route> = self
            .routes
            .into_iter()
            .map(|route| Route {
                path: route.path,
                pool_id: route.pool_id,
            })
            .collect();

        for route in &routes {
            if !upstream_pools.iter().any(|pool| pool.id == route.pool_id) {
                return Err(format!(
                    "Route '{}' references unknown upstream pool '{}'",
                    route.path, route.pool_id
                ));
            }
        }

        config.new_upstream_pools(upstream_pools);
        config.set_routes(routes);

        Ok(config)
    }
}

impl Config {
    fn new() -> Self {
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
            max_header_size: 16384,
            routes: vec![],
        }
    }

    // Upstream management

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
        self.upstream_pools
            .iter()
            .find(|p| p.id == pool_id)
            .and_then(|pool| pool.upstreams.iter().find(|u| u.id == upstream_id))
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

    // Proxy address and header size

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

    // Route management

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
