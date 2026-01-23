//! Horsea δ - Dragon Frontiers #31

pub const SET: &str = "DF";
pub const NUMBER: u32 = 31;
pub const NAME: &str = "Horsea δ";

pub const ATTACK_1_NAME: &str = "Tackle";
pub const ATTACK_1_DAMAGE: u32 = 10;

pub const ATTACK_2_NAME: &str = "Reverse Thrust";
pub const ATTACK_2_DAMAGE: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 31);
        assert_eq!(NAME, "Horsea δ");
    }
}

pub const ATTACK_2_TEXT: &str = "Switch Horsea with 1 of your Benched Pokémon.";
