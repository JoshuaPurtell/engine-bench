//! Copycat - Dragon Frontiers #73

pub const SET: &str = "DF";
pub const NUMBER: u32 = 73;
pub const NAME: &str = "Copycat";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 73);
        assert_eq!(NAME, "Copycat");
    }
}
