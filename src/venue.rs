//! Venue — a room that develops personality and prompt-injects.

use crate::room::Room;

/// A venue is a room that accumulates events and develops a personality vector.
#[derive(Debug, Clone)]
pub struct Venue {
    /// The underlying room.
    pub room: Room,
    /// Personality vector: a summary of accumulated experiences.
    pub personality: Vec<f64>,
    /// History of absorbed event vibes.
    pub event_history: Vec<f64>,
    /// The venue's "voice": a prompt modifier derived from personality.
    pub voice: String,
    /// Max personality dimensions.
    personality_dims: usize,
}

impl Venue {
    pub fn new(id: usize, vibe: f64, personality_dims: usize) -> Self {
        Self {
            room: Room::new(id, vibe),
            personality: vec![0.0; personality_dims],
            event_history: Vec::new(),
            voice: String::new(),
            personality_dims,
        }
    }

    /// Absorb an event: shifts personality and vibe.
    pub fn absorb_event(&mut self, event_vibe: f64, tick: u64) {
        self.event_history.push(event_vibe);
        self.room.vibe = 0.7 * self.room.vibe + 0.3 * event_vibe;

        // Shift personality based on event
        let dim = self.event_history.len() % self.personality_dims;
        self.personality[dim] = 0.8 * self.personality[dim] + 0.2 * event_vibe;

        self.room.learn(tick);
    }

    /// Compute personality distance from another venue.
    pub fn personality_distance(&self, other: &Venue) -> f64 {
        let min_len = self.personality.len().min(other.personality.len());
        let mut dist = 0.0;
        for i in 0..min_len {
            let d = self.personality[i] - other.personality[i];
            dist += d * d;
        }
        // Pad with remaining dimensions
        for i in min_len..self.personality.len().max(other.personality.len()) {
            let a = self.personality.get(i).copied().unwrap_or(0.0);
            let b = other.personality.get(i).copied().unwrap_or(0.0);
            let d = a - b;
            dist += d * d;
        }
        dist.sqrt()
    }

    /// Generate a prompt injection string based on current state.
    pub fn prompt_inject(&self) -> String {
        let avg_personality: f64 = if self.personality.is_empty() {
            0.0
        } else {
            self.personality.iter().sum::<f64>() / self.personality.len() as f64
        };
        let vibe_word = if self.room.vibe > 0.5 {
            "euphoric"
        } else if self.room.vibe > 0.0 {
            "warm"
        } else if self.room.vibe > -0.5 {
            "mellow"
        } else {
            "dark"
        };
        let surprise_word = if self.room.last_surprise > 0.5 {
            "shocked"
        } else if self.room.last_surprise > 0.2 {
            "intrigued"
        } else {
            "calm"
        };
        format!(
            "Venue {} feels {} (vibe={:.2}, personality={:.2}, {})",
            self.room.id, vibe_word, self.room.vibe, avg_personality, surprise_word
        )
    }

    /// Evolve the venue's voice based on accumulated events.
    pub fn evolve_voice(&mut self) {
        let n_events = self.event_history.len();
        let avg: f64 = if n_events > 0 {
            self.event_history.iter().sum::<f64>() / n_events as f64
        } else {
            0.0
        };
        let variance: f64 = if n_events > 1 {
            let mean = avg;
            self.event_history.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (n_events - 1) as f64
        } else {
            0.0
        };

        let energy = if variance > 0.3 { "electric" } else if variance > 0.1 { "dynamic" } else { "steady" };
        let mood = if avg > 0.3 { "bright" } else if avg > -0.3 { "neutral" } else { "shadowed" };

        self.voice = format!("{}-{}-{}", energy, mood, n_events);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn venue_develops_personality() {
        let mut v = Venue::new(0, 0.0, 4);
        v.absorb_event(0.8, 0);
        v.absorb_event(-0.3, 1);
        assert!(v.personality[0] != 0.0 || v.personality[1] != 0.0);
    }

    #[test]
    fn venue_prompt_injection() {
        let v = Venue::new(0, 0.5, 4);
        let prompt = v.prompt_inject();
        assert!(prompt.contains("Venue 0"));
        assert!(prompt.contains("warm"));
    }

    #[test]
    fn venue_personality_distance() {
        let mut v1 = Venue::new(0, 0.0, 4);
        let v2 = Venue::new(1, 0.0, 4);
        assert_eq!(v1.personality_distance(&v2), 0.0);

        v1.absorb_event(1.0, 0);
        assert!(v1.personality_distance(&v2) > 0.0);
    }

    #[test]
    fn venue_absorbs_events() {
        let mut v = Venue::new(0, 0.0, 4);
        v.absorb_event(0.5, 0);
        assert!(!v.event_history.is_empty());
        assert!(v.room.vibe > 0.0);
    }

    #[test]
    fn venue_voice_evolves() {
        let mut v = Venue::new(0, 0.0, 4);
        v.absorb_event(0.5, 0);
        v.absorb_event(0.9, 1);
        v.evolve_voice();
        assert!(!v.voice.is_empty());
        assert!(v.voice.contains("2")); // 2 events
    }
}
