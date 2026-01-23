//! Boost Energy - Dragon Frontiers #87

pub const SET: &str = "DF";
pub const NUMBER: u32 = 87;
pub const NAME: &str = "Boost Energy";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 87);
        assert_eq!(NAME, "Boost Energy");
    }
}
