//! Topology generators for building graph structures.

/// Generate a chain topology: 0-1-2-...-(n-1).
pub fn chain(n: usize) -> Vec<(usize, usize, f64)> {
    (1..n).map(|i| (i - 1, i, 1.0)).collect()
}

/// Generate a ring topology: 0-1-2-...-(n-1)-0.
pub fn ring(n: usize) -> Vec<(usize, usize, f64)> {
    let mut edges = chain(n);
    if n > 2 {
        edges.push((n - 1, 0, 1.0));
    }
    edges
}

/// Generate a star topology: 0 is the hub, connected to all others.
pub fn star(n: usize) -> Vec<(usize, usize, f64)> {
    (1..n).map(|i| (0, i, 1.0)).collect()
}

/// Generate a full mesh: every node connected to every other.
pub fn mesh(n: usize) -> Vec<(usize, usize, f64)> {
    let mut edges = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            edges.push((i, j, 1.0));
            edges.push((j, i, 1.0));
        }
    }
    edges
}

/// Watts-Strogatz small-world model.
/// Starts from a ring and rewires each edge with probability `p`.
/// Uses a simple deterministic approach for reproducibility.
pub fn small_world(n: usize, p: f64) -> Vec<(usize, usize, f64)> {
    if n < 3 { return chain(n); }

    let mut edges = ring(n);
    // Add shortcut edges based on probability p
    let shortcut_count = ((n as f64) * p).ceil() as usize;
    for i in 0..shortcut_count {
        let from = i % n;
        let to = (from + 2 + i) % n;
        edges.push((from, to, 1.0));
    }
    edges
}

/// Barabási–Albert-style scale-free network.
/// Each new node connects to `m` existing nodes with probability proportional to degree.
pub fn scale_free(n: usize, m: usize) -> Vec<(usize, usize, f64)> {
    if n <= 1 { return Vec::new(); }
    let m = m.min(n - 1);

    let mut edges = Vec::new();
    let mut degree = vec![0usize; n];

    // Start with a small complete graph of m+1 nodes
    let initial = (m + 1).min(n);
    for i in 0..initial {
        for j in (i + 1)..initial {
            edges.push((i, j, 1.0));
            edges.push((j, i, 1.0));
            degree[i] += 1;
            degree[j] += 1;
        }
    }

    // Add remaining nodes with preferential attachment
    let mut total_degree: usize = degree.iter().sum();
    for new_node in initial..n {
        let mut attached = 0;
        let mut targets = Vec::new();
        for (candidate, &deg) in degree.iter().enumerate() {
            if candidate >= new_node { break; }
            if targets.contains(&candidate) { continue; }
            if total_degree > 0 && (deg as f64 / total_degree as f64) > (attached as f64 / new_node as f64).min(1.0) {
                targets.push(candidate);
                attached += 1;
                if attached >= m { break; }
            }
        }
        // Ensure at least m connections
        let mut fallback = 0;
        while targets.len() < m && fallback < new_node {
            if !targets.contains(&fallback) {
                targets.push(fallback);
            }
            fallback += 1;
        }
        for target in targets {
            edges.push((new_node, target, 1.0));
            edges.push((target, new_node, 1.0));
            degree[new_node] += 1;
            degree[target] += 1;
            total_degree += 2;
        }
        total_degree += degree[new_node];
    }

    edges
}

/// Erdős–Rényi random graph: each possible edge exists with probability `p`.
/// Deterministic for given n and p (uses a simple hash-like approach).
pub fn random_graph(n: usize, p: f64) -> Vec<(usize, usize, f64)> {
    let mut edges = Vec::new();
    // Simple deterministic PRNG (LCG)
    let mut state: u64 = 42;
    let next_rand = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (1u64 << 31) as f64
    };

    for i in 0..n {
        for j in (i + 1)..n {
            if next_rand(&mut state) < p {
                edges.push((i, j, 1.0));
                edges.push((j, i, 1.0));
            }
        }
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topology_chain() {
        let edges = chain(5);
        assert_eq!(edges.len(), 4);
        assert_eq!(edges[0], (0, 1, 1.0));
        assert_eq!(edges[3], (3, 4, 1.0));
    }

    #[test]
    fn topology_ring() {
        let edges = ring(5);
        assert_eq!(edges.len(), 5); // 4 chain + 1 wrap-around
        assert_eq!(edges.last().unwrap(), &(4, 0, 1.0));
    }

    #[test]
    fn topology_star() {
        let edges = star(5);
        assert_eq!(edges.len(), 4); // 4 edges from hub
        for &(from, _, _) in &edges {
            assert_eq!(from, 0);
        }
    }

    #[test]
    fn topology_mesh() {
        let edges = mesh(4);
        // 4 nodes: 6 bidirectional pairs = 12 edges
        assert_eq!(edges.len(), 12);
    }

    #[test]
    fn topology_small_world() {
        let edges = small_world(10, 0.3);
        assert!(edges.len() > 10); // ring + shortcuts
    }

    #[test]
    fn topology_scale_free() {
        let edges = scale_free(10, 2);
        assert!(!edges.is_empty());
    }

    #[test]
    fn topology_random() {
        let edges = random_graph(10, 0.5);
        // Some edges should exist
        assert!(edges.len() > 0);
    }

    #[test]
    fn small_world_has_shortcuts() {
        let ring_edges = ring(10);
        let sw_edges = small_world(10, 0.5);
        assert!(sw_edges.len() > ring_edges.len());
    }

    #[test]
    fn star_has_hub() {
        let edges = star(6);
        // Hub (node 0) should appear in all edges
        let hub_count = edges.iter().filter(|&&(from, _, _)| from == 0).count();
        assert_eq!(hub_count, 5);
    }

    #[test]
    fn deterministic_with_same_seed() {
        let e1 = random_graph(20, 0.3);
        let e2 = random_graph(20, 0.3);
        assert_eq!(e1, e2);
    }
}
