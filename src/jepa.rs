//! Pluggable JEPA (Joint Embedding Predictive Architecture) predictor trait.

/// A JEPA predictor learns to predict future vibe states from history.
pub trait JepaPredictor {
    /// Feed a new observation (tick, vibe).
    fn observe(&mut self, tick: u64, vibe: f64);

    /// Predict the vibe at a future tick given current state.
    fn predict(&self, current_vibe: f64, horizon: u64) -> f64;

    /// Compute surprise: how unexpected was the last observation?
    fn surprise(&self) -> f64;

    /// Clone into a boxed trait object.
    fn clone_box(&self) -> Box<dyn JepaPredictor>;
}

impl Clone for Box<dyn JepaPredictor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// A simple weighted-history predictor that exponentially decays past observations.
#[derive(Debug, Clone)]
pub struct WeightedHistory {
    weight: f64,
    history: Vec<(u64, f64)>,
    last_prediction: f64,
    last_observation: f64,
    max_history: usize,
}

impl WeightedHistory {
    pub fn new(decay: f64) -> Self {
        Self {
            weight: decay,
            history: Vec::new(),
            last_prediction: 0.0,
            last_observation: 0.0,
            max_history: 100,
        }
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }
}

impl Default for WeightedHistory {
    fn default() -> Self {
        Self::new(0.9)
    }
}

impl JepaPredictor for WeightedHistory {
    fn observe(&mut self, tick: u64, vibe: f64) {
        self.last_observation = vibe;
        self.history.push((tick, vibe));
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        // Update prediction as weighted average
        let mut sum = 0.0;
        let mut total_weight = 0.0;
        let len = self.history.len();
        for (i, &(_, v)) in self.history.iter().enumerate() {
            let age = (len - 1 - i) as f64;
            let w = self.weight.powf(age);
            sum += v * w;
            total_weight += w;
        }
        self.last_prediction = if total_weight > 0.0 { sum / total_weight } else { 0.0 };
    }

    fn predict(&self, current_vibe: f64, _horizon: u64) -> f64 {
        if self.history.is_empty() {
            return current_vibe;
        }
        // Blend current vibe with historical prediction
        0.5 * current_vibe + 0.5 * self.last_prediction
    }

    fn surprise(&self) -> f64 {
        (self.last_observation - self.last_prediction).abs()
    }

    fn clone_box(&self) -> Box<dyn JepaPredictor> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jepa_trait_object_works() {
        let predictor: Box<dyn JepaPredictor> = Box::new(WeightedHistory::new(0.9));
        let prediction = predictor.predict(0.5, 10);
        assert!(!prediction.is_nan());
    }

    #[test]
    fn weighted_history_learns() {
        let mut wh = WeightedHistory::new(0.9);
        for i in 0..10u64 {
            wh.observe(i, 0.5);
        }
        let pred = wh.predict(0.5, 5);
        assert!((pred - 0.5).abs() < 0.1);
    }

    #[test]
    fn weighted_history_surprise() {
        let mut wh = WeightedHistory::new(0.9);
        wh.observe(0, 0.0);
        wh.observe(1, 0.0);
        wh.observe(2, 1.0); // big surprise
        assert!(wh.surprise() > 0.5);
    }

    #[test]
    fn weighted_history_clones() {
        let mut wh = WeightedHistory::new(0.9);
        wh.observe(0, 0.5);
        let cloned: Box<dyn JepaPredictor> = wh.clone_box();
        assert!((cloned.surprise() - wh.surprise()).abs() < f64::EPSILON);
    }
}
