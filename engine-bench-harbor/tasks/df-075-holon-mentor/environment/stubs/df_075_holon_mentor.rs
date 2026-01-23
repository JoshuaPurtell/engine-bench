//! Holon Mentor - Dragon Frontiers #75

pub const SET: &str = "DF";
pub const NUMBER: u32 = 75;
pub const NAME: &str = "Holon Mentor";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 75);
        assert_eq!(NAME, "Holon Mentor");
    }
}
