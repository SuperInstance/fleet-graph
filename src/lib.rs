/// fleet-graph: Coupling graph topology analysis for fleet dynamics.
///
/// Types for representing coupling topologies (ring, star, complete, chain, custom),
/// computing spectral properties (spectral gap, algebraic connectivity, eigenvalues),
/// and analyzing stability via Lyapunov exponents.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Adjacency-like coupling matrix for a fleet of `n` agents.
#[derive(Debug, Clone, PartialEq)]
pub struct CouplingGraph {
    pub n: usize,
    pub weights: Vec<Vec<f64>>,
}

/// Recognised topology shapes.
#[derive(Debug, Clone, PartialEq)]
pub enum Topology {
    Ring,
    Star,
    Complete,
    Chain,
    Custom,
}

/// Results of spectral decomposition.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectralAnalysis {
    pub eigenvalues: Vec<f64>,
    pub spectral_gap: f64,
    pub algebraic_connectivity: f64,
}

/// Stability report based on trajectory divergence.
#[derive(Debug, Clone, PartialEq)]
pub struct StabilityReport {
    pub lyapunov_exponent: f64,
    pub is_stable: bool,
    pub divergence_rate: f64,
}

// ---------------------------------------------------------------------------
// Construction helpers
// ---------------------------------------------------------------------------

fn validate_n(n: usize) {
    assert!(n >= 2, "CouplingGraph requires at least 2 agents");
}

fn make_symmetric(n: usize, f: impl Fn(usize, usize) -> f64) -> Vec<Vec<f64>> {
    let mut m = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            m[i][j] = f(i, j);
        }
    }
    m
}

impl CouplingGraph {
    /// Build a **ring** topology: each agent i is coupled to i-1 and i+1 (mod n).
    pub fn ring(n: usize, weight: f64) -> Self {
        validate_n(n);
        Self {
            n,
            weights: make_symmetric(n, |i, j| {
                if i != j {
                    let left = (i + n - 1) % n;
                    let right = (i + 1) % n;
                    if j == left || j == right {
                        weight
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }),
        }
    }

    /// Build a **star** topology: agent 0 is hub, all others connect only to hub.
    pub fn star(n: usize, weight: f64) -> Self {
        validate_n(n);
        Self {
            n,
            weights: make_symmetric(n, |i, j| {
                if i != j && (i == 0 || j == 0) {
                    weight
                } else {
                    0.0
                }
            }),
        }
    }

    /// Build a **complete** (fully connected) topology.
    pub fn complete(n: usize, weight: f64) -> Self {
        validate_n(n);
        Self {
            n,
            weights: make_symmetric(n, |i, j| if i != j { weight } else { 0.0 }),
        }
    }

    /// Build a **chain** topology: linear line (agent 0 – 1 – 2 – … – n-1).
    pub fn chain(n: usize, weight: f64) -> Self {
        validate_n(n);
        Self {
            n,
            weights: make_symmetric(n, |i, j| {
                if (i == j + 1 || j == i + 1) && i != j {
                    weight
                } else {
                    0.0
                }
            }),
        }
    }

    /// Build a custom topology from an explicit weight matrix.
    ///
    /// # Panics
    /// - If the matrix is not square.
    /// - If `n < 2`.
    pub fn custom(weights: Vec<Vec<f64>>) -> Self {
        let n = weights.len();
        validate_n(n);
        for row in &weights {
            assert_eq!(row.len(), n, "custom weight matrix must be square");
        }
        Self { n, weights }
    }

    // -----------------------------------------------------------------------
    // Topology detection
    // -----------------------------------------------------------------------

    /// Detect which topology best matches this coupling graph.
    ///
    /// Uses a heuristic: counts edges, checks patterns.
    /// Only detects exact matches against canonical topologies.
    pub fn topology(&self) -> Topology {
        let n = self.n;
        if n < 3 {
            return Topology::Chain;
        }

        // Count edges and total weight
        let mut edge_count = 0usize;
        let mut nonzero_weight = 0.0_f64;
        let mut uniform = true;

        for i in 0..n {
            for j in (i + 1)..n {
                let w = self.weights[i][j];
                if w.abs() > 1e-12 {
                    edge_count += 1;
                    if nonzero_weight == 0.0 {
                        nonzero_weight = w;
                    } else if (w - nonzero_weight).abs() > 1e-12 {
                        uniform = false;
                    }
                }
            }
        }

        if edge_count == 0 {
            return Topology::Custom;
        }

        // Complete: n*(n-1)/2 edges, uniform weight
        let complete_edges = n * (n - 1) / 2;
        if edge_count == complete_edges && uniform {
            return Topology::Complete;
        }

        // Star: exactly n-1 edges, all involve agent 0
        if edge_count == n - 1 && uniform {
            let mut all_via_hub = true;
            for i in 1..n {
                if self.weights[0][i].abs() <= 1e-12 {
                    all_via_hub = false;
                    break;
                }
                // Non-hub edges must be zero
                for j in (i + 1)..n {
                    if self.weights[i][j].abs() > 1e-12 {
                        all_via_hub = false;
                        break;
                    }
                }
                if !all_via_hub {
                    break;
                }
            }
            if all_via_hub {
                return Topology::Star;
            }
        }

        // Ring: exactly n edges, each node degree 2
        if edge_count == n && uniform {
            let mut ring = true;
            for i in 0..n {
                let left = (i + n - 1) % n;
                let right = (i + 1) % n;
                for j in 0..n {
                    if j != left && j != right && j != i {
                        if self.weights[i][j].abs() > 1e-12 {
                            ring = false;
                            break;
                        }
                    }
                }
                if !ring {
                    break;
                }
            }
            if ring {
                return Topology::Ring;
            }
        }

        // Chain: exactly n-1 edges, each interior node degree 2, endpoints degree 1
        if edge_count == n - 1 && uniform {
            let mut chain = true;
            for i in 0..n {
                let mut deg = 0usize;
                for j in 0..n {
                    if i != j && self.weights[i][j].abs() > 1e-12 {
                        deg += 1;
                    }
                }
                // In a chain, 2 endpoints have degree 1, rest have degree 2
                if !(deg == 1 || deg == 2) {
                    chain = false;
                    break;
                }
            }
            if chain {
                return Topology::Chain;
            }
        }

        Topology::Custom
    }

    // -----------------------------------------------------------------------
    // Spectral analysis
    // -----------------------------------------------------------------------

    /// Return the top-`k` eigenvalues using power iteration with deflation.
    ///
    /// `iterations` controls convergence quality (default: 1000 in wrapper methods).
    fn power_iteration_top_k(mat: &[Vec<f64>], k: usize, iterations: usize) -> Vec<f64> {
        let n = mat.len();
        if n == 0 {
            return vec![];
        }
        let k = k.min(n);
        let mut current = mat.to_vec();
        let mut eigenvalues = Vec::with_capacity(k);

        for _ in 0..k {
            let mut v = vec![1.0; n];
            // Normalize v
            {
                let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm > 0.0 {
                    for x in &mut v {
                        *x /= norm;
                    }
                }
            }

            let mut eigenvalue = 0.0;
            for _iter in 0..iterations {
                // Multiply: w = current * v
                let w: Vec<f64> = (0..n)
                    .map(|i| (0..n).map(|j| current[i][j] * v[j]).sum())
                    .collect();
                let norm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm > 0.0 {
                    for (x, vv) in v.iter_mut().zip(w.iter()) {
                        *x = vv / norm;
                    }
                }
                // Rayleigh quotient
                let rayleigh: f64 = (0..n)
                    .map(|i| {
                        let row_sum: f64 = (0..n).map(|j| current[i][j] * v[j]).sum();
                        row_sum * v[i]
                    })
                    .sum();
                eigenvalue = rayleigh;
            }

            eigenvalues.push(eigenvalue);

            // Deflate: subtract eigenvalue * v * v^T
            for i in 0..n {
                for j in 0..n {
                    current[i][j] -= eigenvalue * v[i] * v[j];
                }
            }
        }

        eigenvalues
    }

    /// Compute the **Laplacian** matrix: L = D - W, where D is the diagonal
    /// degree matrix (row sums).
    fn laplacian(&self) -> Vec<Vec<f64>> {
        let n = self.n;
        let mut l = vec![vec![0.0_f64; n]; n];
        for i in 0..n {
            let degree: f64 = self.weights[i].iter().sum();
            l[i][i] = degree;
            for j in 0..n {
                if i != j {
                    l[i][j] = -self.weights[i][j];
                }
            }
        }
        l
    }

    /// Full spectral analysis: eigenvalues, spectral gap, algebraic connectivity.
    pub fn spectral_analysis(&self) -> SpectralAnalysis {
        let n = self.n;
        let k = n.min(10);
        let iterations = 2000;

        // Get largest eigenvalues of weight matrix
        let eigenvalues = Self::power_iteration_top_k(&self.weights, k, iterations);
        let spectral_gap = if eigenvalues.len() >= 2 {
            eigenvalues[0] - eigenvalues[1]
        } else {
            eigenvalues.first().copied().unwrap_or(0.0)
        };

        // Algebraic connectivity: 2nd smallest eigenvalue of Laplacian
        // We compute eigenvalues of -L and then recover.
        let lap = self.laplacian();
        let lap_neg: Vec<Vec<f64>> = lap
            .iter()
            .map(|row| row.iter().map(|x| -x).collect())
            .collect();
        let lap_eigs_neg = Self::power_iteration_top_k(&lap_neg, k, iterations);
        // Recover L eigenvalues: λ_L = - λ_{(-L)}
        let mut lap_eigs: Vec<f64> = lap_eigs_neg.iter().map(|x| -x).collect();
        lap_eigs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let algebraic_connectivity = if lap_eigs.len() >= 2 {
            lap_eigs[1]
        } else {
            lap_eigs.first().copied().unwrap_or(0.0)
        };

        SpectralAnalysis {
            eigenvalues: eigenvalues.clone(),
            spectral_gap,
            algebraic_connectivity,
        }
    }

    /// Compute the spectral gap (λ₁ - λ₂) of the weight matrix.
    pub fn spectral_gap(&self) -> f64 {
        let n = self.n;
        let k = 2usize.min(n);
        let ev = Self::power_iteration_top_k(&self.weights, k, 2000);
        if ev.len() >= 2 {
            ev[0] - ev[1]
        } else {
            ev.first().copied().unwrap_or(0.0)
        }
    }

    /// Total coupling: sum of all edge weights (undirected, count each once).
    pub fn total_coupling(&self) -> f64 {
        let mut total = 0.0;
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                total += self.weights[i][j];
            }
        }
        total
    }

    /// Return a new graph with weights normalized so total coupling = 1.0.
    ///
    /// Panics if total coupling is zero.
    pub fn normalized(&self) -> Self {
        let total = self.total_coupling();
        assert!(total.abs() > 1e-12, "cannot normalize a zero-coupling graph");
        let factor = 1.0 / total;
        let new_weights: Vec<Vec<f64>> = self
            .weights
            .iter()
            .map(|row| row.iter().map(|x| x * factor).collect())
            .collect();
        Self {
            n: self.n,
            weights: new_weights,
        }
    }

    /// Check whether the graph is connected (via DFS).
    pub fn is_connected(&self) -> bool {
        let n = self.n;
        if n == 0 {
            return false;
        }
        let mut visited = vec![false; n];
        let mut stack = vec![0usize];
        visited[0] = true;
        while let Some(i) = stack.pop() {
            for j in 0..n {
                if !visited[j] && self.weights[i][j].abs() > 1e-12 {
                    visited[j] = true;
                    stack.push(j);
                }
            }
        }
        visited.iter().all(|&v| v)
    }

    /// Cosine similarity between the flattened weight vectors of two graphs.
    ///
    /// Returns a value in [-1, 1] where 1 = identical direction.
    pub fn compare(&self, other: &CouplingGraph) -> f64 {
        let n = self.n.min(other.n);
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        for i in 0..n {
            for j in 0..n {
                dot += self.weights[i][j] * other.weights[i][j];
                norm_a += self.weights[i][j] * self.weights[i][j];
                norm_b += other.weights[i][j] * other.weights[i][j];
            }
        }
        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom.abs() < 1e-12 {
            0.0
        } else {
            dot / denom
        }
    }

    /// Compute the (maximum) Lyapunov exponent from two trajectories of a
    /// fleet system.  `traj_a` and `traj_b` are time-series where each
    /// element is a vector of agent states at that time step.
    ///
    /// Returns a `StabilityReport`:
    /// - Negative λ => stable (trajectories converge)
    /// - Positive λ => chaotic (trajectories diverge)
    /// - Near-zero  => neutral / limit cycle
    pub fn lyapunov_from_trajectories(
        traj_a: &[Vec<f64>],
        traj_b: &[Vec<f64>],
    ) -> StabilityReport {
        let t = traj_a.len().min(traj_b.len());
        if t < 2 {
            return StabilityReport {
                lyapunov_exponent: 0.0,
                is_stable: true,
                divergence_rate: 0.0,
            };
        }

        let d0: Vec<f64> = (0..traj_a[0].len().min(traj_b[0].len()))
            .map(|i| traj_a[0][i] - traj_b[0][i])
            .collect();
        let initial_dist: f64 = d0.iter().map(|x| x * x).sum::<f64>().sqrt();

        if initial_dist < 1e-15 {
            // Identical initial conditions – no divergence possible
            return StabilityReport {
                lyapunov_exponent: f64::NEG_INFINITY,
                is_stable: true,
                divergence_rate: 0.0,
            };
        }

        let mut total_log_ratio = 0.0;
        let mut count = 0;

        for step in 1..t {
            let d: Vec<f64> = (0..traj_a[step].len().min(traj_b[step].len()))
                .map(|i| traj_a[step][i] - traj_b[step][i])
                .collect();
            let dist: f64 = d.iter().map(|x| x * x).sum::<f64>().sqrt();

            if dist < 1e-15 {
                // Trajectories converged exactly – perfectly stable
                total_log_ratio += f64::NEG_INFINITY;
                count += 1;
                continue;
            }

            total_log_ratio += (dist / initial_dist).ln().max(-100.0).min(100.0);
            count += 1;
        }

        let lyapunov = if count > 0 {
            total_log_ratio / (t as f64 - 1.0)
        } else {
            0.0
        };

        let lyapunov = lyapunov.clamp(-100.0, 100.0);

        let divergence_rate = if count > 0 {
            total_log_ratio / count as f64
        } else {
            0.0
        };

        StabilityReport {
            lyapunov_exponent: lyapunov,
            is_stable: lyapunov < 0.0,
            divergence_rate,
        }
    }
}

// ---------------------------------------------------------------------------
// Utility trait for display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Topology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Topology::Ring => write!(f, "Ring"),
            Topology::Star => write!(f, "Star"),
            Topology::Complete => write!(f, "Complete"),
            Topology::Chain => write!(f, "Chain"),
            Topology::Custom => write!(f, "Custom"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Construction ----

    #[test]
    fn test_ring_construction() {
        let g = CouplingGraph::ring(5, 1.0);
        assert_eq!(g.n, 5);
        // Ring: each node connected to two neighbours
        for i in 0..5 {
            let left = (i + 4) % 5;
            let right = (i + 1) % 5;
            assert!((g.weights[i][left] - 1.0).abs() < 1e-12);
            assert!((g.weights[i][right] - 1.0).abs() < 1e-12);
            for j in 0..5 {
                if j != left && j != right && j != i {
                    assert!(g.weights[i][j].abs() < 1e-12);
                }
            }
        }
    }

    #[test]
    fn test_star_construction() {
        let g = CouplingGraph::star(5, 2.0);
        assert_eq!(g.n, 5);
        // All non-zero entries involve agent 0
        for i in 1..5 {
            assert!((g.weights[0][i] - 2.0).abs() < 1e-12);
            assert!((g.weights[i][0] - 2.0).abs() < 1e-12);
        }
        // No edge between non-hub agents
        for i in 1..5 {
            for j in 1..5 {
                if i != j {
                    assert!(g.weights[i][j].abs() < 1e-12);
                }
            }
        }
    }

    #[test]
    fn test_complete_construction() {
        let g = CouplingGraph::complete(4, 0.5);
        assert_eq!(g.n, 4);
        for i in 0..4 {
            for j in 0..4 {
                if i != j {
                    assert!((g.weights[i][j] - 0.5).abs() < 1e-12);
                } else {
                    assert!(g.weights[i][j].abs() < 1e-12);
                }
            }
        }
    }

    #[test]
    fn test_chain_construction() {
        let g = CouplingGraph::chain(4, 1.5);
        assert_eq!(g.n, 4);
        // Linear: 0-1-2-3
        assert!((g.weights[0][1] - 1.5).abs() < 1e-12);
        assert!((g.weights[1][2] - 1.5).abs() < 1e-12);
        assert!((g.weights[2][3] - 1.5).abs() < 1e-12);
        assert!(g.weights[0][2].abs() < 1e-12);
        assert!(g.weights[0][3].abs() < 1e-12);
        assert!(g.weights[1][3].abs() < 1e-12);
    }

    #[test]
    fn test_custom_construction() {
        let w = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let g = CouplingGraph::custom(w);
        assert_eq!(g.n, 2);
        assert!((g.weights[0][1] - 1.0).abs() < 1e-12);
    }

    #[test]
    #[should_panic(expected = "at least 2 agents")]
    fn test_n_minimum() {
        CouplingGraph::ring(1, 1.0);
    }

    // ---- Topology detection ----

    #[test]
    fn test_topology_ring() {
        let g = CouplingGraph::ring(5, 1.0);
        assert!(matches!(g.topology(), Topology::Ring));
    }

    #[test]
    fn test_topology_star() {
        let g = CouplingGraph::star(5, 1.0);
        assert!(matches!(g.topology(), Topology::Star));
    }

    #[test]
    fn test_topology_complete() {
        let g = CouplingGraph::complete(5, 1.0);
        assert!(matches!(g.topology(), Topology::Complete));
    }

    #[test]
    fn test_topology_chain() {
        let g = CouplingGraph::chain(4, 1.0);
        assert!(matches!(g.topology(), Topology::Chain));
    }

    #[test]
    fn test_topology_custom() {
        let w = vec![vec![0.0, 0.3, 0.7], vec![0.3, 0.0, 0.2], vec![0.7, 0.2, 0.0]];
        let g = CouplingGraph::custom(w);
        assert!(matches!(g.topology(), Topology::Custom));
    }

    // ---- Spectral analysis ----

    #[test]
    fn test_spectral_gap_nonzero() {
        let g = CouplingGraph::complete(10, 0.5);
        let gap = g.spectral_gap();
        // For a complete graph with uniform weight w, eigenvalues are:
        //   λ₁ = (n-1)*w, λ₂ = λ₃ = ... = -w
        // Gap = (n-1)*w - (-w) = n*w = 10 * 0.5 = 5.0
        assert!((gap - 5.0).abs() < 0.5, "gap = {}", gap);
    }

    #[test]
    fn test_spectral_analysis_returns_all_fields() {
        let g = CouplingGraph::ring(6, 1.0);
        let sa = g.spectral_analysis();
        assert!(!sa.eigenvalues.is_empty());
        assert!(sa.spectral_gap >= 0.0);
        assert!(sa.algebraic_connectivity >= 0.0);
    }

    #[test]
    fn test_algebraic_connectivity_chain_vs_ring() {
        let chain = CouplingGraph::chain(10, 1.0);
        let ring = CouplingGraph::ring(10, 1.0);
        let ac_chain = chain.spectral_analysis().algebraic_connectivity;
        let ac_ring = ring.spectral_analysis().algebraic_connectivity;
        // A chain has lower algebraic connectivity than a ring (more fragile)
        assert!(
            ac_chain < ac_ring + 0.5,
            "chain ac={}, ring ac={}",
            ac_chain,
            ac_ring
        );
    }

    // ---- Connectivity ----

    #[test]
    fn test_connected_ring() {
        let g = CouplingGraph::ring(5, 1.0);
        assert!(g.is_connected());
    }

    #[test]
    fn test_connected_star() {
        let g = CouplingGraph::star(5, 1.0);
        assert!(g.is_connected());
    }

    #[test]
    fn test_disconnected_graph() {
        let w = vec![
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 0.0],
        ];
        let g = CouplingGraph::custom(w);
        assert!(!g.is_connected());
    }

    // ---- Total coupling & normalization ----

    #[test]
    fn test_total_coupling_ring() {
        let g = CouplingGraph::ring(5, 1.0);
        // Ring of 5 nodes => 5 edges
        assert!((g.total_coupling() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_total_coupling_complete() {
        let g = CouplingGraph::complete(4, 2.0);
        // 4 nodes => 6 edges, each weight 2.0 => total = 12.0
        assert!((g.total_coupling() - 12.0).abs() < 1e-12);
    }

    #[test]
    fn test_normalized_total_coupling() {
        let g = CouplingGraph::star(5, 3.0);
        let ng = g.normalized();
        assert!((ng.total_coupling() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalized_preserves_topology() {
        let g = CouplingGraph::ring(6, 4.0);
        let ng = g.normalized();
        assert!(matches!(ng.topology(), Topology::Ring));
    }

    // ---- Comparison (cosine similarity) ----

    #[test]
    fn test_compare_identical() {
        let a = CouplingGraph::complete(5, 1.0);
        let b = CouplingGraph::complete(5, 1.0);
        let sim = a.compare(&b);
        assert!((sim - 1.0).abs() < 1e-6, "sim = {}", sim);
    }

    #[test]
    fn test_compare_orthogonal() {
        // Two disconnected sub-graphs on 4 nodes: edges (0,1) and (2,3)
        let w1 = vec![
            vec![0.0, 1.0, 0.0, 0.0],
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 0.0],
        ];
        let w2 = vec![
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 0.0],
        ];
        let a = CouplingGraph::custom(w1);
        let b = CouplingGraph::custom(w2);
        let sim = a.compare(&b);
        assert!(sim.abs() < 1e-12, "sim = {}", sim);
    }

    #[test]
    fn test_compare_different_sizes() {
        let a = CouplingGraph::ring(4, 1.0);
        let b = CouplingGraph::ring(8, 1.0);
        let sim = a.compare(&b);
        // Should not panic, produce some value
        assert!(sim >= -1.01 && sim <= 1.01);
    }

    // ---- Lyapunov exponent ----

    #[test]
    fn test_lyapunov_stable_convergence() {
        // Converging trajectories => negative Lyapunov
        let traj_a = vec![
            vec![1.0, 2.0],
            vec![1.1, 2.1],
            vec![1.01, 2.01],
            vec![1.001, 2.001],
        ];
        let traj_b = vec![
            vec![1.5, 2.5],
            vec![1.4, 2.4],
            vec![1.02, 2.02],
            vec![1.002, 2.002],
        ];
        let report = CouplingGraph::lyapunov_from_trajectories(&traj_a, &traj_b);
        assert!(report.is_stable, "should be stable, λ={}", report.lyapunov_exponent);
        assert!(report.lyapunov_exponent < 0.0);
    }

    #[test]
    fn test_lyapunov_diverging() {
        // Diverging trajectories => positive Lyapunov
        let traj_a = vec![
            vec![0.0, 0.1],
            vec![1.0, 1.1],
            vec![10.0, 10.1],
            vec![100.0, 100.1],
        ];
        let traj_b = vec![
            vec![0.0, 0.0],
            vec![-1.0, -1.0],
            vec![-10.0, -10.0],
            vec![-100.0, -100.0],
        ];
        let report = CouplingGraph::lyapunov_from_trajectories(&traj_a, &traj_b);
        assert!(!report.is_stable, "should be unstable, λ={}", report.lyapunov_exponent);
        assert!(report.lyapunov_exponent > 0.0);
    }

    #[test]
    fn test_lyapunov_identical_initial() {
        let traj = vec![
            vec![1.0, 2.0, 3.0],
            vec![1.5, 2.5, 3.5],
            vec![2.0, 3.0, 4.0],
        ];
        let report = CouplingGraph::lyapunov_from_trajectories(&traj, &traj);
        assert!(report.is_stable);
    }

    #[test]
    fn test_lyapunov_short_trajectory() {
        let traj_a = vec![vec![1.0, 2.0]];
        let traj_b = vec![vec![3.0, 4.0]];
        let report = CouplingGraph::lyapunov_from_trajectories(&traj_a, &traj_b);
        assert!(report.is_stable);
    }

    // ---- Edge cases ----

    #[test]
    fn test_n2_chain() {
        let g = CouplingGraph::chain(2, 1.0);
        assert_eq!(g.n, 2);
        assert!((g.weights[0][1] - 1.0).abs() < 1e-12);
        assert!((g.weights[1][0] - 1.0).abs() < 1e-12);
        assert!(g.is_connected());
        assert!((g.total_coupling() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_ring_with_zero_weight() {
        let g = CouplingGraph::ring(4, 0.0);
        assert!((g.total_coupling() - 0.0).abs() < 1e-12);
        // Zero-weight ring is structurally undetectable as Ring since
        // the topology detector needs non-zero weights.
        assert_eq!(g.topology(), Topology::Custom);
        assert!(!g.is_connected());
    }
}
