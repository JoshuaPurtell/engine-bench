use tcg_rules_ex::SpecialCondition;
//! Arbok δ - Dragon Frontiers #13

pub const SET: &str = "DF";
pub const NUMBER: u32 = 13;
pub const NAME: &str = "Arbok δ";

pub const ATTACK_1_NAME: &str = "Burning Venom";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Strangle";
pub const ATTACK_2_DAMAGE: u32 = 50;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 13);
        assert_eq!(NAME, "Arbok δ");
    }
}

// Condition helper for Burning Venom
pub fn burning_venom_condition() -> SpecialCondition {
    // TODO: Implement
    SpecialCondition::Burned
}

pub const ATTACK_1_TEXT: &str = "The Defending Pokémon is now Burned and Poisoned.";
pub const ATTACK_2_TEXT: &str = "If the Defending Pokémon has δ on its card, this attack does 50 damage plus 30 more damage.";
