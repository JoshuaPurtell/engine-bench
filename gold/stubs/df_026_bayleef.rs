use tcg_rules_ex::SpecialCondition;
//! Bayleef δ - Dragon Frontiers #26

pub const SET: &str = "DF";
pub const NUMBER: u32 = 26;
pub const NAME: &str = "Bayleef δ";

pub const ATTACK_1_NAME: &str = "Poisonpowder";
pub const ATTACK_1_DAMAGE: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 26);
        assert_eq!(NAME, "Bayleef δ");
    }
}

// Condition helper for Poisonpowder
pub fn poisonpowder_condition() -> SpecialCondition {
    // TODO: Implement
    SpecialCondition::Poisoned
}

pub const ATTACK_1_TEXT: &str = "The Defending Pokémon is now Poisoned.";
