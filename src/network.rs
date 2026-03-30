//! Social networks — graph models, metrics, small-world properties.

use serde::{Deserialize, Serialize};

use crate::error::{Result, SanghaError};

/// A social network represented as an adjacency list with weighted edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SocialNetwork {
    /// Number of nodes.
    pub node_count: usize,
    /// Adjacency list: `edges[i]` contains `(neighbor, weight)` pairs.
    pub edges: Vec<Vec<(usize, f64)>>,
}

impl SocialNetwork {
    /// Create an empty network with `n` nodes.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            node_count: n,
            edges: (0..n).map(|_| Vec::new()).collect(),
        }
    }

    /// Add an undirected edge between nodes `a` and `b` with given weight.
    ///
    /// # Errors
    ///
    /// Returns [`SanghaError::InvalidNetwork`] if node indices are out of bounds.
    pub fn add_edge(&mut self, a: usize, b: usize, weight: f64) -> Result<()> {
        if a >= self.node_count || b >= self.node_count {
            return Err(SanghaError::InvalidNetwork(
                "node index out of bounds".into(),
            ));
        }
        self.edges[a].push((b, weight));
        if a != b {
            self.edges[b].push((a, weight));
        }
        Ok(())
    }

    /// Degree of a node (number of connections).
    ///
    /// # Errors
    ///
    /// Returns [`SanghaError::InvalidNetwork`] if node index is out of bounds.
    #[must_use = "returns the degree without side effects"]
    pub fn degree(&self, node: usize) -> Result<usize> {
        if node >= self.node_count {
            return Err(SanghaError::InvalidNetwork(
                "node index out of bounds".into(),
            ));
        }
        Ok(self.edges[node].len())
    }

    /// Average degree across all nodes.
    #[inline]
    #[must_use]
    pub fn average_degree(&self) -> f64 {
        if self.node_count == 0 {
            return 0.0;
        }
        let total: usize = self.edges.iter().map(|e| e.len()).sum();
        total as f64 / self.node_count as f64
    }

    /// Total number of edges (each undirected edge counted once).
    #[inline]
    #[must_use]
    pub fn edge_count(&self) -> usize {
        let mut self_loops = 0;
        let total: usize = self
            .edges
            .iter()
            .enumerate()
            .map(|(node, neighbors)| {
                self_loops += neighbors.iter().filter(|&&(nb, _)| nb == node).count();
                neighbors.len()
            })
            .sum();
        // Undirected edges are stored twice (once per endpoint), self-loops only once.
        (total + self_loops) / 2
    }
}

/// Generate a Watts-Strogatz small-world network with the default seed (42).
///
/// Creates a ring lattice of `n` nodes each connected to `k` nearest neighbors,
/// then rewires each edge with probability `beta`.
///
/// - `beta = 0`: regular lattice (high clustering, long paths)
/// - `beta = 1`: random graph (low clustering, short paths)
/// - `beta ~ 0.1`: small-world (high clustering, short paths)
///
/// # Errors
///
/// Returns [`SanghaError::InvalidNetwork`] if `n < 4`, `k` is odd, or `k >= n`.
#[must_use = "returns the generated network without side effects"]
pub fn watts_strogatz(n: usize, k: usize, beta: f64) -> Result<SocialNetwork> {
    watts_strogatz_with_seed(n, k, beta, 42)
}

/// Generate a Watts-Strogatz small-world network with a caller-supplied PRNG seed.
///
/// See [`watts_strogatz`] for parameter details.
///
/// # Errors
///
/// Returns [`SanghaError::InvalidNetwork`] if `n < 4`, `k` is odd, or `k >= n`.
#[must_use = "returns the generated network without side effects"]
pub fn watts_strogatz_with_seed(n: usize, k: usize, beta: f64, seed: u64) -> Result<SocialNetwork> {
    if n < 4 {
        return Err(SanghaError::InvalidNetwork("need at least 4 nodes".into()));
    }
    if !k.is_multiple_of(2) || k == 0 {
        return Err(SanghaError::InvalidNetwork("k must be even and > 0".into()));
    }
    if k >= n {
        return Err(SanghaError::InvalidNetwork("k must be less than n".into()));
    }

    let mut net = SocialNetwork::new(n);

    // Create ring lattice
    let half_k = k / 2;
    for i in 0..n {
        for j in 1..=half_k {
            let neighbor = (i + j) % n;
            net.add_edge(i, neighbor, 1.0)?;
        }
    }

    // Rewire edges with probability beta
    // Use a simple LCG pseudo-random for reproducibility
    if beta > 0.0 {
        let mut state: u64 = seed;
        for i in 0..n {
            for j in 1..=half_k {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let rand_val = (state >> 33) as f64 / (u32::MAX as f64);
                if rand_val < beta {
                    let old_neighbor = (i + j) % n;
                    // Pick a random new target
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let new_neighbor = ((state >> 33) as usize) % n;
                    let already_connected = net.edges[i].iter().any(|&(nb, _)| nb == new_neighbor);
                    if new_neighbor != i && new_neighbor != old_neighbor && !already_connected {
                        // Remove old edge and add new one
                        net.edges[i].retain(|&(nb, _)| nb != old_neighbor);
                        net.edges[old_neighbor].retain(|&(nb, _)| nb != i);
                        net.add_edge(i, new_neighbor, 1.0)?;
                    }
                }
            }
        }
    }

    Ok(net)
}

/// Local clustering coefficient for a node.
///
/// Fraction of pairs of neighbors that are themselves connected.
#[must_use = "returns the clustering coefficient without side effects"]
pub fn clustering_coefficient(network: &SocialNetwork, node: usize) -> Result<f64> {
    if node >= network.node_count {
        return Err(SanghaError::InvalidNetwork(
            "node index out of bounds".into(),
        ));
    }
    let neighbors: Vec<usize> = network.edges[node].iter().map(|&(n, _)| n).collect();
    let k = neighbors.len();
    if k < 2 {
        return Ok(0.0);
    }

    let mut triangles = 0;
    for i in 0..k {
        for j in (i + 1)..k {
            let ni = neighbors[i];
            let nj = neighbors[j];
            if network.edges[ni].iter().any(|&(n, _)| n == nj) {
                triangles += 1;
            }
        }
    }

    let possible = k * (k - 1) / 2;
    Ok(triangles as f64 / possible as f64)
}

/// Average clustering coefficient across all nodes.
#[must_use = "returns the average clustering coefficient without side effects"]
pub fn average_clustering_coefficient(network: &SocialNetwork) -> Result<f64> {
    if network.node_count == 0 {
        return Ok(0.0);
    }
    let mut sum = 0.0;
    for i in 0..network.node_count {
        sum += clustering_coefficient(network, i)?;
    }
    Ok(sum / network.node_count as f64)
}

/// Dunbar's number layers: typical social group sizes.
pub const DUNBAR_LAYERS: [usize; 4] = [5, 15, 50, 150];

/// Degree distribution of the network.
#[must_use = "returns the degree distribution without side effects"]
pub fn degree_distribution(network: &SocialNetwork) -> Vec<usize> {
    let mut dist = Vec::new();
    for edges in &network.edges {
        let deg = edges.len();
        if deg >= dist.len() {
            dist.resize(deg + 1, 0);
        }
        dist[deg] += 1;
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_network() {
        let net = SocialNetwork::new(10);
        assert_eq!(net.node_count, 10);
        assert_eq!(net.edge_count(), 0);
        assert!((net.average_degree() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_add_edge() {
        let mut net = SocialNetwork::new(5);
        net.add_edge(0, 1, 1.0).unwrap();
        assert_eq!(net.degree(0).unwrap(), 1);
        assert_eq!(net.degree(1).unwrap(), 1);
        assert_eq!(net.edge_count(), 1);
    }

    #[test]
    fn test_watts_strogatz_regular() {
        // beta=0: regular lattice
        let net = watts_strogatz(20, 4, 0.0).unwrap();
        assert_eq!(net.node_count, 20);
        // Each node should have degree 4 in regular lattice
        for i in 0..20 {
            assert_eq!(net.degree(i).unwrap(), 4);
        }
    }

    #[test]
    fn test_watts_strogatz_clustering() {
        // Regular lattice should have high clustering
        let net = watts_strogatz(20, 4, 0.0).unwrap();
        let cc = average_clustering_coefficient(&net).unwrap();
        assert!(cc > 0.3); // regular lattice has high clustering
    }

    #[test]
    fn test_clustering_coefficient_complete() {
        // Complete graph of 4 nodes: clustering = 1.0
        let mut net = SocialNetwork::new(4);
        for i in 0..4 {
            for j in (i + 1)..4 {
                net.add_edge(i, j, 1.0).unwrap();
            }
        }
        let cc = clustering_coefficient(&net, 0).unwrap();
        assert!((cc - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_degree_distribution() {
        let mut net = SocialNetwork::new(4);
        net.add_edge(0, 1, 1.0).unwrap();
        net.add_edge(0, 2, 1.0).unwrap();
        let dist = degree_distribution(&net);
        // node 0: degree 2, nodes 1,2: degree 1, node 3: degree 0
        assert_eq!(dist[0], 1); // 1 node with degree 0
        assert_eq!(dist[1], 2); // 2 nodes with degree 1
        assert_eq!(dist[2], 1); // 1 node with degree 2
    }

    #[test]
    fn test_dunbar_layers() {
        assert_eq!(DUNBAR_LAYERS, [5, 15, 50, 150]);
    }

    #[test]
    fn test_network_serde_roundtrip() {
        let mut net = SocialNetwork::new(3);
        net.add_edge(0, 1, 0.5).unwrap();
        let json = serde_json::to_string(&net).unwrap();
        let back: SocialNetwork = serde_json::from_str(&json).unwrap();
        assert_eq!(net.node_count, back.node_count);
    }

    #[test]
    fn test_add_edge_out_of_bounds() {
        let mut net = SocialNetwork::new(3);
        assert!(net.add_edge(0, 5, 1.0).is_err());
    }

    #[test]
    fn test_self_loop_edge_count() {
        let mut net = SocialNetwork::new(3);
        net.add_edge(0, 0, 1.0).unwrap(); // self-loop
        net.add_edge(0, 1, 1.0).unwrap(); // normal edge
        assert_eq!(net.edge_count(), 2);
    }

    #[test]
    fn test_watts_strogatz_with_seed_deterministic() {
        let net1 = super::watts_strogatz_with_seed(20, 4, 0.3, 123).unwrap();
        let net2 = super::watts_strogatz_with_seed(20, 4, 0.3, 123).unwrap();
        assert_eq!(net1.edge_count(), net2.edge_count());
        for i in 0..20 {
            assert_eq!(net1.degree(i).unwrap(), net2.degree(i).unwrap());
        }
    }

    #[test]
    fn test_watts_strogatz_different_seeds_differ() {
        let net1 = super::watts_strogatz_with_seed(20, 4, 0.5, 1).unwrap();
        let net2 = super::watts_strogatz_with_seed(20, 4, 0.5, 999).unwrap();
        // With beta=0.5 and different seeds, degree distributions should differ
        let dist1 = super::degree_distribution(&net1);
        let dist2 = super::degree_distribution(&net2);
        assert_ne!(dist1, dist2);
    }

    #[test]
    fn test_watts_strogatz_error_too_few_nodes() {
        assert!(watts_strogatz(3, 2, 0.0).is_err());
    }

    #[test]
    fn test_watts_strogatz_error_odd_k() {
        assert!(watts_strogatz(10, 3, 0.0).is_err());
    }

    #[test]
    fn test_watts_strogatz_error_k_ge_n() {
        assert!(watts_strogatz(6, 6, 0.0).is_err());
    }

    #[test]
    fn test_watts_strogatz_no_multi_edges() {
        // With high beta, rewiring shouldn't create duplicate edges
        let net = watts_strogatz_with_seed(10, 4, 1.0, 42).unwrap();
        for (node, neighbors) in net.edges.iter().enumerate() {
            let mut seen = std::collections::HashSet::new();
            for &(nb, _) in neighbors {
                assert!(seen.insert(nb) || nb == node, "duplicate edge {node}->{nb}");
            }
        }
    }

    #[test]
    fn test_clustering_coefficient_isolated() {
        let net = SocialNetwork::new(5);
        let cc = clustering_coefficient(&net, 0).unwrap();
        assert!((cc - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_clustering_coefficient_out_of_bounds() {
        let net = SocialNetwork::new(3);
        assert!(clustering_coefficient(&net, 5).is_err());
    }

    #[test]
    fn test_average_clustering_empty() {
        let net = SocialNetwork::new(0);
        let cc = average_clustering_coefficient(&net).unwrap();
        assert!((cc - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_degree_distribution_empty() {
        let net = SocialNetwork::new(0);
        let dist = degree_distribution(&net);
        assert!(dist.is_empty());
    }

    #[test]
    fn test_degree_out_of_bounds() {
        let net = SocialNetwork::new(3);
        assert!(net.degree(5).is_err());
    }

    #[test]
    fn test_zero_node_network() {
        let net = SocialNetwork::new(0);
        assert_eq!(net.node_count, 0);
        assert_eq!(net.edge_count(), 0);
        assert!((net.average_degree() - 0.0).abs() < 1e-10);
    }
}
