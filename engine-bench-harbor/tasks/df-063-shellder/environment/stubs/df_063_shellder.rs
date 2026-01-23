use tcg_rules_ex::SpecialCondition;
//! Shellder δ - Dragon Frontiers #63

pub const SET: &str = "DF";
pub const NUMBER: u32 = 63;
pub const NAME: &str = "Shellder δ";

pub const ATTACK_1_NAME: &str = "Shell Grab";
pub const ATTACK_1_DAMAGE: u32 = 10;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 63);
        assert_eq!(NAME, "Shellder δ");
    }
}

// Condition helper for Shell Grab
pub fn shell_grab_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Paralyzed) } else { None }
}

pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Paralyzed.";
