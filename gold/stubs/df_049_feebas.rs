//! Feebas δ - Dragon Frontiers #49

pub const SET: &str = "DF";
pub const NUMBER: u32 = 49;
pub const NAME: &str = "Feebas δ";

pub const ATTACK_1_NAME: &str = "Flail";
pub const ATTACK_1_DAMAGE: u32 = 10;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 49);
        assert_eq!(NAME, "Feebas δ");
    }
}

// Attack helper for Flail
pub fn flail_damage(damage_counters: u32) -> i32 {
    // TODO: Implement
    (10 * damage_counters) as i32
}

pub const ATTACK_1_TEXT: &str = "Does 10 damage times the number of damage counters on Feebas.";
