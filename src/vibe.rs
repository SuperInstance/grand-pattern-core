//! Mono-dimensional vibe. One number.

/// A mono-dimensional vibe value. Just an f64.
pub type Vibe = f64;

/// The neutral vibe (zero).
pub const NEUTRAL: Vibe = 0.0;

/// Clamp a vibe to [-1.0, 1.0].
pub fn clamp(v: Vibe) -> Vibe {
    v.clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vibe_is_f64() {
        let v: Vibe = 0.42;
        assert!((v - 0.42f64).abs() < f64::EPSILON);
    }

    #[test]
    fn clamp_limits() {
        assert_eq!(clamp(2.0), 1.0);
        assert_eq!(clamp(-3.0), -1.0);
        assert_eq!(clamp(0.5), 0.5);
    }
}
