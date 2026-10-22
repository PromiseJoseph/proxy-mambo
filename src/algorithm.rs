use std::sync::atomic::Ordering;

use crate::types::{LoadBalancingAlgorithm, Upstream, UpstreamPool};

pub fn round_robin(pool: &UpstreamPool) -> Option<&Upstream> {
    if pool.upstreams.is_empty() {
        return None;
    }

    let index = match &pool.algorithm {
        LoadBalancingAlgorithm::RoundRobin { current_index } => {
            current_index.fetch_add(1, Ordering::Relaxed) % pool.upstreams.len()
        }
        _ => return None,
    };

    pool.upstreams.get(index)
}

pub fn use_algorithm(pool: &UpstreamPool) -> Option<&Upstream> {
    match &pool.algorithm {
        LoadBalancingAlgorithm::RoundRobin { .. } => round_robin(pool),
        //more algorithm will be added here..
    }
}
