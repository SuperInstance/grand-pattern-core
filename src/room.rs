//! Rooms — nodes in the cell graph that hold vibe and learn via JEPA.

use crate::jepa::{JepaPredictor, WeightedHistory};
use core::fmt;

/// A room is a node in the cell graph.
pub struct Room {
    /// Room id (index in the graph).
    pub id: usize,
    /// Mono-dimensional vibe value.
    pub vibe: f64,
    /// Pluggable JEPA predictor.
    pub jepa: Box<dyn JepaPredictor>,
    /// Last surprise value.
    pub last_surprise: f64,
}

impl fmt::Debug for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Room")
            .field("id", &self.id)
            .field("vibe", &self.vibe)
            .field("last_surprise", &self.last_surprise)
            .finish_non_exhaustive()
    }
}

impl Room {
    pub fn new(id: usize, vibe: f64) -> Self {
        Self {
            id,
            vibe,
            jepa: Box::new(WeightedHistory::default()),
            last_surprise: 0.0,
        }
    }

    pub fn with_jepa(id: usize, vibe: f64, jepa: Box<dyn JepaPredictor>) -> Self {
        Self {
            id,
            vibe,
            jepa,
            last_surprise: 0.0,
        }
    }

    /// Feed current vibe to JEPA and update surprise.
    pub fn learn(&mut self, tick: u64) {
        self.jepa.observe(tick, self.vibe);
        self.last_surprise = self.jepa.surprise();
    }
}

impl Clone for Room {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            vibe: self.vibe,
            jepa: self.jepa.clone(),
            last_surprise: self.last_surprise,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_has_mono_vibe() {
        let r = Room::new(0, 0.42);
        assert!((r.vibe - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn room_uses_pluggable_jepa() {
        let custom: Box<dyn JepaPredictor> = Box::new(WeightedHistory::new(0.5).with_max_history(10));
        let r = Room::with_jepa(0, 0.0, custom);
        assert_eq!(r.id, 0);
    }

    #[test]
    fn room_learns() {
        let mut r = Room::new(0, 0.0);
        r.learn(0); // observe 0.0, surprise = 0
        r.vibe = 1.0;
        r.learn(1); // observe 1.0, prediction was ~0.5, surprise > 0
        assert!(r.last_surprise > 0.0, "surprise should be nonzero after sudden change");
    }
}
