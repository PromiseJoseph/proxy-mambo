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
