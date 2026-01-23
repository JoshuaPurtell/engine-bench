use tcg_rules_ex::SpecialCondition;
//! Flaaffy δ - Dragon Frontiers #30

pub const SET: &str = "DF";
pub const NUMBER: u32 = 30;
pub const NAME: &str = "Flaaffy δ";

pub const ATTACK_1_NAME: &str = "Thundershock";
pub const ATTACK_1_DAMAGE: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 30);
        assert_eq!(NAME, "Flaaffy δ");
    }
}

// Condition helper for Thundershock
pub fn thundershock_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Paralyzed) } else { None }
}

pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Paralyzed.";
