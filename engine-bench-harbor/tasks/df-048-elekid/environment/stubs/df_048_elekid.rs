//! Elekid δ - Dragon Frontiers #48

pub const SET: &str = "DF";
pub const NUMBER: u32 = 48;
pub const NAME: &str = "Elekid δ";

pub const ABILITY_1_NAME: &str = "Baby Evolution";

pub const ATTACK_1_NAME: &str = "Thunder Spear";
pub const ATTACK_1_DAMAGE: u32 = 0;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 48);
        assert_eq!(NAME, "Elekid δ");
    }
}

// Helper for thunder_spear_snipe_damage
pub fn thunder_spear_snipe_damage() -> u32 {
    // TODO: Implement
    10
}

// Helper for thunder_spear_snipe_scope
pub fn thunder_spear_snipe_scope() -> &'static str {
    // TODO: Implement
    "OppBench"
}

pub const ABILITY_1_TEXT: &str = "Once during your turn (before your attack), you may put Electabuzz from your hand onto Elekid (this counts as evolving Elekid) and remove all damage counters from Elekid.";
pub const ATTACK_1_TEXT: &str = "Choose 1 of your opponent's Pokémon. This attack does 10 damage to that Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)";
