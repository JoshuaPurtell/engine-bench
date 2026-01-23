use tcg_rules_ex::SpecialCondition;
//! Dragonair δ - Dragon Frontiers #28

pub const SET: &str = "DF";
pub const NUMBER: u32 = 28;
pub const NAME: &str = "Dragonair δ";

pub const ATTACK_1_NAME: &str = "Wrap";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Horn Attack";
pub const ATTACK_2_DAMAGE: u32 = 40;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 28);
        assert_eq!(NAME, "Dragonair δ");
    }
}

// Condition helper for Wrap
pub fn wrap_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Paralyzed) } else { None }
}

pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Paralyzed.";
