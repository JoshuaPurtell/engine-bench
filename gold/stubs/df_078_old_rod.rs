//! Old Rod - Dragon Frontiers #78

pub const SET: &str = "DF";
pub const NUMBER: u32 = 78;
pub const NAME: &str = "Old Rod";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 78);
        assert_eq!(NAME, "Old Rod");
    }
}
