use tcg_rules_ex::SpecialCondition;
//! Gligar δ - Dragon Frontiers #16

pub const SET: &str = "DF";
pub const NUMBER: u32 = 16;
pub const NAME: &str = "Gligar δ";

pub const ATTACK_1_NAME: &str = "Sting Turn";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Tail Sting";
pub const ATTACK_2_DAMAGE: u32 = 10;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 16);
        assert_eq!(NAME, "Gligar δ");
    }
}

// Condition helper for Sting Turn
pub fn sting_turn_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Paralyzed) } else { None }
}

// Condition helper for Tail Sting
pub fn tail_sting_condition() -> SpecialCondition {
    // TODO: Implement
    SpecialCondition::Poisoned
}

pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Paralyzed and switch Gligar with 1 of your Benched Pokémon.";
pub const ATTACK_2_TEXT: &str = "If the Defending Pokémon is Pokémon-ex, this attack does 10 damage plus 10 more damage and the Defending Pokémon is now Poisoned.";
