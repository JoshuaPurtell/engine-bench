//! Seel δ - Dragon Frontiers #62

pub const SET: &str = "DF";
pub const NUMBER: u32 = 62;
pub const NAME: &str = "Seel δ";

pub const ATTACK_1_NAME: &str = "Pound";
pub const ATTACK_1_DAMAGE: u32 = 10;

pub const ATTACK_2_NAME: &str = "Aurora Beam";
pub const ATTACK_2_DAMAGE: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 62);
        assert_eq!(NAME, "Seel δ");
    }
}
