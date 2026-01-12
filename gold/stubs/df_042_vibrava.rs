//! Vibrava δ - Dragon Frontiers #42

pub const SET: &str = "DF";
pub const NUMBER: u32 = 42;
pub const NAME: &str = "Vibrava δ";

pub const ATTACK_1_NAME: &str = "Bite";
pub const ATTACK_1_DAMAGE: u32 = 20;

pub const ATTACK_2_NAME: &str = "Sonic Noise";
pub const ATTACK_2_DAMAGE: u32 = 30;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 42);
        assert_eq!(NAME, "Vibrava δ");
    }
}

pub const ATTACK_2_TEXT: &str = "If the Defending Pokémon is Pokémon-ex, that Pokémon is now Confused.";
