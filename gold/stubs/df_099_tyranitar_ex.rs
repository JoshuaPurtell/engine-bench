//! Tyranitar ex δ - Dragon Frontiers #99

pub const SET: &str = "DF";
pub const NUMBER: u32 = 99;
pub const NAME: &str = "Tyranitar ex δ";

pub const ATTACK_1_NAME: &str = "Electromark";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Hyper Claws";
pub const ATTACK_2_DAMAGE: u32 = 70;

pub const ATTACK_3_NAME: &str = "Shock-wave";
pub const ATTACK_3_DAMAGE: u32 = 0;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 99);
        assert_eq!(NAME, "Tyranitar ex δ");
    }
}

pub const ATTACK_1_TEXT: &str = "Put a Shock-wave marker on 1 of your opponent's Pokémon.";
pub const ATTACK_2_TEXT: &str = "If the Defending Pokémon is a Stage 2 Evolved Pokémon, this attack does 70 damage plus 20 more damage.";
pub const ATTACK_3_TEXT: &str = "Choose 1 of your opponent's Pokémon that has any Shock-wave markers on it. That Pokémon is Knocked Out.";
