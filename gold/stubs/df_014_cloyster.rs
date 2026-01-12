//! Cloyster δ - Dragon Frontiers #14

pub const SET: &str = "DF";
pub const NUMBER: u32 = 14;
pub const NAME: &str = "Cloyster δ";

pub const ABILITY_1_NAME: &str = "Solid Shell";

pub const ATTACK_1_NAME: &str = "Grind";
pub const ATTACK_1_DAMAGE: u32 = 10;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 14);
        assert_eq!(NAME, "Cloyster δ");
    }
}

// Helper for grind_per_energy_damage
pub fn grind_per_energy_damage(attached_energy: u32) -> i32 {
    // TODO: Implement
    10 + (10 * attached_energy) as i32
}

pub const ABILITY_1_TEXT: &str = "Prevent all effects of attacks, including damage, done by your opponent's Pokémon to each of your Benched Pokémon that has δ on its card.";
pub const ATTACK_1_TEXT: &str = "Does 10 damage plus 10 more damage for each Energy attached to Cloyster.";
