//! Conservation laws — verify vibe mass is preserved.

use crate::room::Room;

/// Compute total vibe mass across rooms.
pub fn check(rooms: &[Option<Room>]) -> f64 {
    rooms.iter().filter_map(|r| r.as_ref()).map(|r| r.vibe).sum()
}

/// Verify that vibe mass is conserved within tolerance.
pub fn verify(before: f64, after: f64, tolerance: f64) -> bool {
    (before - after).abs() <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::Room;

    #[test]
    fn check_total_vibe() {
        let rooms = vec![
            Some(Room::new(0, 0.5)),
            Some(Room::new(1, -0.3)),
            Some(Room::new(2, 0.8)),
        ];
        let total = check(&rooms);
        assert!((total - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn verify_conservation_passes() {
        assert!(verify(1.0, 1.0, 0.01));
        assert!(verify(1.0, 1.005, 0.01));
    }

    #[test]
    fn verify_conservation_fails() {
        assert!(!verify(1.0, 2.0, 0.01));
    }

    #[test]
    fn conservation_with_empty() {
        let rooms: Vec<Option<Room>> = vec![];
        assert_eq!(check(&rooms), 0.0);
    }
}
