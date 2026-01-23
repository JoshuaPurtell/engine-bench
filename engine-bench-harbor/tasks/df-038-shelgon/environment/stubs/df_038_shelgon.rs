//! Shelgon δ - Dragon Frontiers #38

pub const SET: &str = "DF";
pub const NUMBER: u32 = 38;
pub const NAME: &str = "Shelgon δ";

pub const ATTACK_1_NAME: &str = "Headbutt";
pub const ATTACK_1_DAMAGE: u32 = 20;

pub const ATTACK_2_NAME: &str = "Double-edge";
pub const ATTACK_2_DAMAGE: u32 = 50;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 38);
        assert_eq!(NAME, "Shelgon δ");
    }
}

// Self-damage helper for Double-edge
pub fn double_edge_self_damage() -> u32 {
    // TODO: Implement
    10
}

pub const ATTACK_2_TEXT: &str = "Shelgon does 10 damage to itself.";
