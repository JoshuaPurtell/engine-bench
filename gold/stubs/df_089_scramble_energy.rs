//! Scramble Energy - Dragon Frontiers #89

pub const SET: &str = "DF";
pub const NUMBER: u32 = 89;
pub const NAME: &str = "Scramble Energy";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 89);
        assert_eq!(NAME, "Scramble Energy");
    }
}
