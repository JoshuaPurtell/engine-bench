use tcg_rules_ex::SpecialCondition;
//! Ralts - Dragon Frontiers #60

pub const SET: &str = "DF";
pub const NUMBER: u32 = 60;
pub const NAME: &str = "Ralts";

pub const ATTACK_1_NAME: &str = "Hypnosis";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Psychic Boom";
pub const ATTACK_2_DAMAGE: u32 = 10;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 60);
        assert_eq!(NAME, "Ralts");
    }
}

// Attack helper for Psychic Boom
pub fn psychic_boom_damage(attached_energy: u32) -> i32 {
    // TODO: Implement
    (10 * attached_energy) as i32
}

// Condition helper for Hypnosis
pub fn hypnosis_condition() -> SpecialCondition {
    // TODO: Implement
    SpecialCondition::Asleep
}

pub const ATTACK_1_TEXT: &str = "The Defending Pokémon is now Asleep.";
pub const ATTACK_2_TEXT: &str = "Does 10 damage times the amount of Energy attached to the Defending Pokémon.";
