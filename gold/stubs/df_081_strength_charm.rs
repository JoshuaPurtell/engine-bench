//! Strength Charm - Dragon Frontiers #81

pub const SET: &str = "DF";
pub const NUMBER: u32 = 81;
pub const NAME: &str = "Strength Charm";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 81);
        assert_eq!(NAME, "Strength Charm");
    }
}
