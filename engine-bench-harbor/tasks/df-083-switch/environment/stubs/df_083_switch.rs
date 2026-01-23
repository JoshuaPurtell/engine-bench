//! Switch - Dragon Frontiers #83

pub const SET: &str = "DF";
pub const NUMBER: u32 = 83;
pub const NAME: &str = "Switch";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 83);
        assert_eq!(NAME, "Switch");
    }
}
