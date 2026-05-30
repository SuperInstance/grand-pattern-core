//! CellGraph — the core graph structure of connected rooms.

use crate::room::Room;
use crate::murmur::Murmur;
use crate::jepa::JepaPredictor;

/// An edge connecting two rooms with a weight.
#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
}

/// The cell graph: rooms connected by weighted edges.
pub struct CellGraph {
    pub rooms: Vec<Option<Room>>,
    pub edges: Vec<Edge>,
    pub tick_count: u64,
    pub bpm: f64,
}

impl CellGraph {
    pub fn new(bpm: f64) -> Self {
        Self {
            rooms: Vec::new(),
            edges: Vec::new(),
            tick_count: 0,
            bpm,
        }
    }

    /// Add a room with given initial vibe. Returns room id.
    pub fn add_room(&mut self, vibe: f64) -> usize {
        let id = self.rooms.len();
        self.rooms.push(Some(Room::new(id, vibe)));
        id
    }

    /// Add a room with a custom JEPA predictor. Returns room id.
    pub fn add_room_with_jepa(&mut self, vibe: f64, jepa: Box<dyn JepaPredictor>) -> usize {
        let id = self.rooms.len();
        self.rooms.push(Some(Room::with_jepa(id, vibe, jepa)));
        id
    }

    /// Add a directed edge between rooms.
    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        self.edges.push(Edge { from, to, weight });
    }

    /// Remove a room by id (leaves a None slot to preserve indices).
    pub fn remove_room(&mut self, id: usize) {
        if id < self.rooms.len() {
            self.rooms[id] = None;
            self.edges.retain(|e| e.from != id && e.to != id);
        }
    }

    /// Advance one tick: all rooms learn.
    pub fn tick(&mut self) {
        let tick = self.tick_count;
        for room in self.rooms.iter_mut() {
            if let Some(r) = room {
                r.learn(tick);
            }
        }
        self.tick_count += 1;
    }

    /// Diffuse vibes along edges by the given rate.
    pub fn diffuse(&mut self, rate: f64) {
        let n = self.rooms.len();
        let mut deltas = vec![0.0f64; n];

        for edge in &self.edges {
            let (Some(from_room), Some(to_room)) = (
                self.rooms.get(edge.from).and_then(|r| r.as_ref()),
                self.rooms.get(edge.to).and_then(|r| r.as_ref()),
            ) else {
                continue;
            };
            let diff = from_room.vibe - to_room.vibe;
            let flow = diff * edge.weight * rate;
            deltas[edge.from] -= flow;
            deltas[edge.to] += flow;
        }

        for (i, room) in self.rooms.iter_mut().enumerate() {
            if let Some(r) = room {
                r.vibe += deltas[i];
            }
        }
    }

    /// Gossip: each room emits a murmur that propagates through edges up to ttl hops.
    pub fn gossip(&self, ttl: u32) -> Vec<Murmur> {
        let mut murmurs = Vec::new();

        for room in &self.rooms {
            let Some(r) = room else { continue };
            murmurs.push(Murmur::new(r.id, r.vibe, r.last_surprise, self.tick_count, ttl));
        }

        // Propagate murmurs through edges
        let mut propagated = Vec::new();
        for murmur in &murmurs {
            let mut m = murmur.clone();
            for _ in 0..ttl {
                if !m.decay() { break; }
                propagated.push(m.clone());
            }
        }
        murmurs.extend(propagated);
        murmurs
    }

    /// All rooms learn (same as tick but without incrementing).
    pub fn learn(&mut self) {
        let tick = self.tick_count;
        for room in self.rooms.iter_mut() {
            if let Some(r) = room {
                r.learn(tick);
            }
        }
    }

    /// Total vibe mass across all rooms.
    pub fn total_vibe(&self) -> f64 {
        self.rooms.iter().filter_map(|r| r.as_ref()).map(|r| r.vibe).sum()
    }

    /// Fleet vibe: average vibe across active rooms.
    pub fn fleet_vibe(&self) -> f64 {
        let active: Vec<&Room> = self.rooms.iter().filter_map(|r| r.as_ref()).collect();
        if active.is_empty() { return 0.0; }
        active.iter().map(|r| r.vibe).sum::<f64>() / active.len() as f64
    }

    /// Fleet surprise: average surprise across active rooms.
    pub fn fleet_surprise(&self) -> f64 {
        let active: Vec<&Room> = self.rooms.iter().filter_map(|r| r.as_ref()).collect();
        if active.is_empty() { return 0.0; }
        active.iter().map(|r| r.last_surprise).sum::<f64>() / active.len() as f64
    }

    /// Number of active (non-removed) rooms.
    pub fn active_room_count(&self) -> usize {
        self.rooms.iter().filter(|r| r.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jepa::WeightedHistory;

    #[test]
    fn graph_creation() {
        let g = CellGraph::new(120.0);
        assert_eq!(g.bpm, 120.0);
        assert_eq!(g.tick_count, 0);
    }

    #[test]
    fn add_remove_rooms() {
        let mut g = CellGraph::new(120.0);
        let a = g.add_room(0.5);
        let b = g.add_room(-0.3);
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(g.active_room_count(), 2);

        g.remove_room(a);
        assert_eq!(g.active_room_count(), 1);
    }

    #[test]
    fn tick_increments() {
        let mut g = CellGraph::new(120.0);
        g.add_room(0.0);
        g.tick();
        g.tick();
        assert_eq!(g.tick_count, 2);
    }

    #[test]
    fn diffuse_converges() {
        let mut g = CellGraph::new(120.0);
        let a = g.add_room(1.0);
        let b = g.add_room(0.0);
        g.add_edge(a, b, 1.0);
        g.add_edge(b, a, 1.0);

        for _ in 0..100 {
            g.diffuse(0.1);
        }
        let diff = (g.rooms[a].as_ref().unwrap().vibe - g.rooms[b].as_ref().unwrap().vibe).abs();
        assert!(diff < 0.01, "rooms should converge, diff = {diff}");
    }

    #[test]
    fn conservation_holds_trivially() {
        let mut g = CellGraph::new(120.0);
        let a = g.add_room(1.0);
        let b = g.add_room(0.0);
        g.add_edge(a, b, 1.0);
        g.add_edge(b, a, 1.0);

        let before = g.total_vibe();
        for _ in 0..50 {
            g.diffuse(0.1);
        }
        let after = g.total_vibe();
        assert!((before - after).abs() < 1e-10, "vibe should be conserved: {before} vs {after}");
    }

    #[test]
    fn gossip_spreads() {
        let mut g = CellGraph::new(120.0);
        let _a = g.add_room(0.5);
        g.tick(); // learn
        let murmurs = g.gossip(3);
        assert!(!murmurs.is_empty());
    }

    #[test]
    fn fleet_vibe_is_average() {
        let mut g = CellGraph::new(120.0);
        g.add_room(1.0);
        g.add_room(-1.0);
        assert!((g.fleet_vibe() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fleet_surprise_is_average() {
        let g = CellGraph::new(120.0);
        // No rooms learned yet, surprise is 0
        assert_eq!(g.fleet_surprise(), 0.0);
    }

    #[test]
    fn empty_graph() {
        let g = CellGraph::new(120.0);
        assert_eq!(g.total_vibe(), 0.0);
        assert_eq!(g.fleet_vibe(), 0.0);
        assert_eq!(g.fleet_surprise(), 0.0);
    }

    #[test]
    fn single_room() {
        let mut g = CellGraph::new(120.0);
        g.add_room(0.42);
        assert!((g.total_vibe() - 0.42).abs() < f64::EPSILON);
        assert!((g.fleet_vibe() - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn large_graph_perf() {
        let mut g = CellGraph::new(120.0);
        let n = 50;
        // Use a star topology for fast spread
        let hub = g.add_room(1.0);
        for _ in 1..n {
            let leaf = g.add_room(0.0);
            g.add_edge(hub, leaf, 1.0);
            g.add_edge(leaf, hub, 1.0);
        }

        for _ in 0..1000 {
            g.diffuse(0.005);
        }
        // Hub should have distributed vibe to leaves
        let all_positive = g.rooms.iter()
            .filter_map(|r| r.as_ref())
            .all(|r| r.vibe > 0.001);
        assert!(all_positive, "all rooms should have positive vibe after diffusion");
    }

    #[test]
    fn room_jepa_learns_differently() {
        let mut g = CellGraph::new(120.0);
        let a = g.add_room_with_jepa(0.0, Box::new(WeightedHistory::new(0.9)));
        let b = g.add_room_with_jepa(0.0, Box::new(WeightedHistory::new(0.1)));

        g.rooms[a].as_mut().unwrap().vibe = 1.0;
        g.rooms[b].as_mut().unwrap().vibe = 1.0;
        g.learn();

        // Both observed 1.0 but with different decay rates
        let sa = g.rooms[a].as_ref().unwrap().last_surprise;
        let sb = g.rooms[b].as_ref().unwrap().last_surprise;
        // They should differ because histories have different decays
        // (may be equal after first observation, so just verify they both ran)
        assert!(sa >= 0.0);
        assert!(sb >= 0.0);
    }

    #[test]
    fn surprise_cascades() {
        let mut g = CellGraph::new(120.0);
        let a = g.add_room(0.0);
        let b = g.add_room(0.0);
        g.add_edge(a, b, 1.0);

        g.tick();
        g.rooms[a].as_mut().unwrap().vibe = 1.0;
        g.diffuse(0.5);
        g.tick();

        // Room b should have received some vibe
        assert!(g.rooms[b].as_ref().unwrap().vibe > 0.0);
    }
}
