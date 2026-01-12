use tcg_rules_ex::SpecialCondition;
//! Xatu δ - Dragon Frontiers #25

pub const SET: &str = "DF";
pub const NUMBER: u32 = 25;
pub const NAME: &str = "Xatu δ";

pub const ABILITY_1_NAME: &str = "Extra Feather";

pub const ATTACK_1_NAME: &str = "Confuse Ray";
pub const ATTACK_1_DAMAGE: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 25);
        assert_eq!(NAME, "Xatu δ");
    }
}

// Condition helper for Confuse Ray
pub fn confuse_ray_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Confused) } else { None }
}

pub const ABILITY_1_TEXT: &str = "Each of your Stage 2 Pokémon-ex does 10 more damage to the Defending Pokémon (before applying Weakness and Resistance).";
pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Confused.";
