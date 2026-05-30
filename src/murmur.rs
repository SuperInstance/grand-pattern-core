//! Murmurs — messages that rooms gossip to each other.

/// A murmur is a transient message propagating through the graph.
#[derive(Debug, Clone)]
pub struct Murmur {
    /// Source room id.
    pub source: usize,
    /// Vibe value being gossiped.
    pub vibe: f64,
    /// Surprise at source when murmur was created.
    pub surprise: f64,
    /// Tick when this murmur was created.
    pub tick: u64,
    /// Time-to-live: hops remaining before expiry.
    pub ttl: u32,
}

impl Murmur {
    pub fn new(source: usize, vibe: f64, surprise: f64, tick: u64, ttl: u32) -> Self {
        Self { source, vibe, surprise, tick, ttl }
    }

    /// Decay the murmur by one hop. Returns false if expired.
    pub fn decay(&mut self) -> bool {
        if self.ttl == 0 {
            return false;
        }
        self.ttl -= 1;
        // Surprise decays proportionally
        self.surprise *= 0.9;
        self.ttl > 0
    }

    /// Check if this murmur is still alive.
    pub fn alive(&self) -> bool {
        self.ttl > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn murmur_decay() {
        let mut m = Murmur::new(0, 0.5, 0.8, 10, 3);
        assert!(m.alive());
        assert!(m.decay()); // ttl goes 3 -> 2
        assert!(m.alive());
        assert!(m.decay()); // 2 -> 1
        assert!(m.alive());
        assert!(!m.decay()); // 1 -> 0
        assert!(!m.alive());
    }

    #[test]
    fn murmur_surprise_decays() {
        let mut m = Murmur::new(0, 0.5, 1.0, 0, 2);
        let s0 = m.surprise;
        m.decay();
        assert!(m.surprise < s0);
    }
}
