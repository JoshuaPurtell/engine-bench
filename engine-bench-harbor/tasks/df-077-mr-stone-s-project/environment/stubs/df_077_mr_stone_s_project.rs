//! Mr. Stone's Project - Dragon Frontiers #77

pub const SET: &str = "DF";
pub const NUMBER: u32 = 77;
pub const NAME: &str = "Mr. Stone's Project";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 77);
        assert_eq!(NAME, "Mr. Stone's Project");
    }
}
