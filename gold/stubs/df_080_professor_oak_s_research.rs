//! Professor Oak's Research - Dragon Frontiers #80

pub const SET: &str = "DF";
pub const NUMBER: u32 = 80;
pub const NAME: &str = "Professor Oak's Research";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 80);
        assert_eq!(NAME, "Professor Oak's Research");
    }
}
