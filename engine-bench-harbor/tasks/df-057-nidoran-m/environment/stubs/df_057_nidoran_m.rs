use tcg_rules_ex::SpecialCondition;
//! Nidoran ♂ δ - Dragon Frontiers #57

pub const SET: &str = "DF";
pub const NUMBER: u32 = 57;
pub const NAME: &str = "Nidoran ♂ δ";

pub const ATTACK_1_NAME: &str = "Peck";
pub const ATTACK_1_DAMAGE: u32 = 10;

pub const ATTACK_2_NAME: &str = "Poison Horn";
pub const ATTACK_2_DAMAGE: u32 = 0;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 57);
        assert_eq!(NAME, "Nidoran ♂ δ");
    }
}

// Condition helper for Poison Horn
pub fn poison_horn_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Poisoned) } else { None }
}

pub const ATTACK_2_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Poisoned.";
