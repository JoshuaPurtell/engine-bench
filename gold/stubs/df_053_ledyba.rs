use tcg_rules_ex::SpecialCondition;
//! Ledyba δ - Dragon Frontiers #53

pub const SET: &str = "DF";
pub const NUMBER: u32 = 53;
pub const NAME: &str = "Ledyba δ";

pub const ATTACK_1_NAME: &str = "Tackle";
pub const ATTACK_1_DAMAGE: u32 = 10;

pub const ATTACK_2_NAME: &str = "Supersonic";
pub const ATTACK_2_DAMAGE: u32 = 0;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 53);
        assert_eq!(NAME, "Ledyba δ");
    }
}

// Condition helper for Supersonic
pub fn supersonic_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Confused) } else { None }
}

pub const ATTACK_2_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Confused.";
