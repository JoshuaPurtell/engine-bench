use tcg_rules_ex::SpecialCondition;
//! Dewgong δ - Dragon Frontiers #15

pub const SET: &str = "DF";
pub const NUMBER: u32 = 15;
pub const NAME: &str = "Dewgong δ";

pub const ABILITY_1_NAME: &str = "Delta Protection";

pub const ATTACK_1_NAME: &str = "Ice Beam";
pub const ATTACK_1_DAMAGE: u32 = 20;

pub const ATTACK_2_NAME: &str = "Surge";
pub const ATTACK_2_DAMAGE: u32 = 40;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 15);
        assert_eq!(NAME, "Dewgong δ");
    }
}

// Attack helper for Surge
pub fn surge_damage(type_energy: u32) -> i32 {
    // TODO: Implement
    if type_energy >= 2 { 40 + 20 } else { 40 }
}

// Condition helper for Ice Beam
pub fn ice_beam_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Paralyzed) } else { None }
}

pub const ABILITY_1_TEXT: &str = "Any damage done to Dewgong by attacks from your opponent's Pokémon that has δ on its card is reduced by 40 (after applying Weakness and Resistance).";
pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Paralyzed.";
pub const ATTACK_2_TEXT: &str = "If Dewgong has at least 2 Water Energy attached to it, this attack does 40 damage plus 20 more damage.";
