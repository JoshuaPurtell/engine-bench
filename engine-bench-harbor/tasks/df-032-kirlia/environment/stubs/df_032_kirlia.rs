use tcg_rules_ex::SpecialCondition;
//! Kirlia - Dragon Frontiers #32

pub const SET: &str = "DF";
pub const NUMBER: u32 = 32;
pub const NAME: &str = "Kirlia";

pub const ATTACK_1_NAME: &str = "Psyshock";
pub const ATTACK_1_DAMAGE: u32 = 20;

pub const ATTACK_2_NAME: &str = "Link Blast";
pub const ATTACK_2_DAMAGE: u32 = 60;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 32);
        assert_eq!(NAME, "Kirlia");
    }
}

// Condition helper for Psyshock
pub fn psyshock_condition(heads: bool) -> Option<SpecialCondition> {
    // TODO: Implement
    if heads { Some(SpecialCondition::Paralyzed) } else { None }
}

pub const ATTACK_1_TEXT: &str = "Flip a coin. If heads, the Defending Pokémon is now Paralyzed.";
pub const ATTACK_2_TEXT: &str = "If Kirlia and the Defending Pokémon have a different amount of Energy attached to them, this attack's base damage is 30 instead of 60.";
