use std::sync::atomic::Ordering;

use crate::types::{LoadBalancingAlgorithm, Upstream, UpstreamPool};

fn round_robin(pool: &UpstreamPool) -> Option<&Upstream> {
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

async fn weighted_round_robin(pool: &UpstreamPool) -> Option<&Upstream> {
    let (weights, current_weights) = match &pool.algorithm {
        LoadBalancingAlgorithm::WeightedRoundRobin {
            weights,
            current_weights,
        } => (weights, current_weights),
        _ => return None,
    };

    if pool.upstreams.is_empty() {
        return None;
    }

    let total_weight: isize = pool
        .upstreams
        .iter()
        .map(|upstream| *weights.get(&upstream.id).unwrap_or(&0) as isize)
        .sum();

    if total_weight <= 0 {
        return None;
    }

    let selected_id = {
        let mut current_weights = current_weights.lock().await;

        let mut selected_id: Option<String> = None;
        let mut highest_weight = isize::MIN;

        for upstream in &pool.upstreams {
            let weight = *weights.get(&upstream.id).unwrap_or(&0) as isize;

            let current = current_weights.entry(upstream.id.clone()).or_insert(0);

            *current += weight;

            if *current > highest_weight {
                highest_weight = *current;
                selected_id = Some(upstream.id.clone());
            }
        }

        let selected_id = selected_id?;

        *current_weights.get_mut(&selected_id)? -= total_weight;

        selected_id
    };

    pool.upstreams
        .iter()
        .find(|upstream| upstream.id == selected_id)
}

pub async fn use_algorithm(pool: &UpstreamPool) -> Option<&Upstream> {
    match &pool.algorithm {
        LoadBalancingAlgorithm::RoundRobin { .. } => round_robin(pool),
        LoadBalancingAlgorithm::WeightedRoundRobin { .. } => weighted_round_robin(pool).await,
    }
}
