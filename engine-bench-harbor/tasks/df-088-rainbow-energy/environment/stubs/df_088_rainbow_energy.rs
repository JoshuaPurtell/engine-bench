//! δ Rainbow Energy - Dragon Frontiers #88

pub const SET: &str = "DF";
pub const NUMBER: u32 = 88;
pub const NAME: &str = "δ Rainbow Energy";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 88);
        assert_eq!(NAME, "δ Rainbow Energy");
    }
}
