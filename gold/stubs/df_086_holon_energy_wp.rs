//! Holon Energy WP - Dragon Frontiers #86

pub const SET: &str = "DF";
pub const NUMBER: u32 = 86;
pub const NAME: &str = "Holon Energy WP";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 86);
        assert_eq!(NAME, "Holon Energy WP");
    }
}
