//! Holon Energy FF - Dragon Frontiers #84

pub const SET: &str = "DF";
pub const NUMBER: u32 = 84;
pub const NAME: &str = "Holon Energy FF";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 84);
        assert_eq!(NAME, "Holon Energy FF");
    }
}
