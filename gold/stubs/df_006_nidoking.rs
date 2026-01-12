//! Nidoking δ - Dragon Frontiers #6

pub const SET: &str = "DF";
pub const NUMBER: u32 = 6;
pub const NAME: &str = "Nidoking δ";

pub const ATTACK_1_NAME: &str = "Linear Attack";
pub const ATTACK_1_DAMAGE: u32 = 0;

pub const ATTACK_2_NAME: &str = "Dark Horn";
pub const ATTACK_2_DAMAGE: u32 = 60;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 6);
        assert_eq!(NAME, "Nidoking δ");
    }
}

// Helper for linear_attack_snipe_damage
pub fn linear_attack_snipe_damage() -> u32 {
    // TODO: Implement
    30
}

// Helper for linear_attack_snipe_scope
pub fn linear_attack_snipe_scope() -> &'static str {
    // TODO: Implement
    "OppBench"
}

pub const ATTACK_1_TEXT: &str = "Choose 1 of your opponent's Pokémon. This attack does 30 damage to that Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)";
pub const ATTACK_2_TEXT: &str = "You may discard a Basic Pokémon or Evolution card from your hand. If you do, choose 1 of your opponent's Benched Pokémon and do 20 damage to that Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)";
