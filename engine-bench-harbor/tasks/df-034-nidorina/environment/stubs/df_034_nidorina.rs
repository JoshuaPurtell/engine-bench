use tcg_rules_ex::SpecialCondition;
//! Nidorina δ - Dragon Frontiers #34

pub const SET: &str = "DF";
pub const NUMBER: u32 = 34;
pub const NAME: &str = "Nidorina δ";

pub const ATTACK_1_NAME: &str = "Poison Sting";
pub const ATTACK_1_DAMAGE: u32 = 10;

pub const ATTACK_2_NAME: &str = "Rear Kick";
pub const ATTACK_2_DAMAGE: u32 = 40;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 34);
        assert_eq!(NAME, "Nidorina δ");
    }
}

// Condition helper for Poison Sting
pub fn poison_sting_condition() -> SpecialCondition {
    // TODO: Implement
    SpecialCondition::Poisoned
}

pub const ATTACK_1_TEXT: &str = "The Defending Pokémon is now Poisoned.";
