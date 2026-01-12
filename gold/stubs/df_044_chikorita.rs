use tcg_rules_ex::SpecialCondition;
//! Chikorita δ - Dragon Frontiers #44

pub const SET: &str = "DF";
pub const NUMBER: u32 = 44;
pub const NAME: &str = "Chikorita δ";

pub const ATTACK_1_NAME: &str = "Sleep Powder";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Tackle";
pub const ATTACK_2_DAMAGE: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 44);
        assert_eq!(NAME, "Chikorita δ");
    }
}

// Condition helper for Sleep Powder
pub fn sleep_powder_condition() -> SpecialCondition {
    // TODO: Implement
    SpecialCondition::Asleep
}

pub const ATTACK_1_TEXT: &str = "The Defending Pokémon is now Asleep.";
