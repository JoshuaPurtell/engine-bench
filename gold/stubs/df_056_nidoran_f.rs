//! Nidoran ♀ δ - Dragon Frontiers #56

pub const SET: &str = "DF";
pub const NUMBER: u32 = 56;
pub const NAME: &str = "Nidoran ♀ δ";

pub const ATTACK_1_NAME: &str = "Tail Whip";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Scratch";
pub const ATTACK_2_DAMAGE: u32 = 10;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 56);
        assert_eq!(NAME, "Nidoran ♀ δ");
    }
}

pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon can't attack during your opponent's next turn.";
