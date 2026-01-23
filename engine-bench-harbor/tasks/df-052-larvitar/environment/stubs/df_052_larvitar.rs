use tcg_rules_ex::SpecialCondition;
//! Larvitar δ - Dragon Frontiers #52

pub const SET: &str = "DF";
pub const NUMBER: u32 = 52;
pub const NAME: &str = "Larvitar δ";

pub const ATTACK_1_NAME: &str = "Paralyzing Gaze";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Horn Attack";
pub const ATTACK_2_DAMAGE: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 52);
        assert_eq!(NAME, "Larvitar δ");
    }
}

// Condition helper for Paralyzing Gaze
pub fn paralyzing_gaze_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Paralyzed) } else { None }
}

pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Paralyzed.";
