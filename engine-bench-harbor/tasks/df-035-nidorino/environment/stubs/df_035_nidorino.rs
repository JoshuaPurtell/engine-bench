//! Nidorino δ - Dragon Frontiers #35

pub const SET: &str = "DF";
pub const NUMBER: u32 = 35;
pub const NAME: &str = "Nidorino δ";

pub const ATTACK_1_NAME: &str = "Rage";
pub const ATTACK_1_DAMAGE: u32 = 10;

pub const ATTACK_2_NAME: &str = "Horn Drill";
pub const ATTACK_2_DAMAGE: u32 = 30;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 35);
        assert_eq!(NAME, "Nidorino δ");
    }
}

// Helper for rage_per_counter_damage
pub fn rage_per_counter_damage(damage_counters: u32) -> i32 {
    // TODO: Implement
    10 + (10 * damage_counters) as i32
}

pub const ATTACK_1_TEXT: &str = "Does 10 damage plus 10 more damage for each damage counter on Nidorino.";
