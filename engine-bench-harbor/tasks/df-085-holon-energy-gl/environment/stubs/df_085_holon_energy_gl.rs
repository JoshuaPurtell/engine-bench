//! Holon Energy GL - Dragon Frontiers #85

pub const SET: &str = "DF";
pub const NUMBER: u32 = 85;
pub const NAME: &str = "Holon Energy GL";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 85);
        assert_eq!(NAME, "Holon Energy GL");
    }
}
