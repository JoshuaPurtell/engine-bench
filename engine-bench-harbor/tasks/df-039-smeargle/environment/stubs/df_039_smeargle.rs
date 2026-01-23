use tcg_rules_ex::SpecialCondition;
//! Smeargle δ - Dragon Frontiers #39

pub const SET: &str = "DF";
pub const NUMBER: u32 = 39;
pub const NAME: &str = "Smeargle δ";

pub const ATTACK_1_NAME: &str = "Collect";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Flickering Tail";
pub const ATTACK_2_DAMAGE: u32 = 10;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 39);
        assert_eq!(NAME, "Smeargle δ");
    }
}

// Condition helper for Flickering Tail
pub fn flickering_tail_condition() -> SpecialCondition {
    // TODO: Implement
    SpecialCondition::Asleep
}

pub const ATTACK_1_TEXT: &str = "Draw a card.";
pub const ATTACK_2_TEXT: &str = "If the Defending Pokémon is Pokémon-ex, this attack does 10 damage plus 10 more damage and the Defending Pokémon is now Asleep.";
