//! Island Hermit - Dragon Frontiers #76

pub const SET: &str = "DF";
pub const NUMBER: u32 = 76;
pub const NAME: &str = "Island Hermit";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 76);
        assert_eq!(NAME, "Island Hermit");
    }
}
