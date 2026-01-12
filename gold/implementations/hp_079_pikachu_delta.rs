//! Pikachu δ - Holon Phantoms #79
pub const SET: &str = "HP";
pub const NUMBER: u32 = 79;
pub const NAME: &str = "Pikachu δ";

pub const TAIL_WHAP_DAMAGE: i32 = 10;
pub const STEEL_HEADBUTT_BASE: i32 = 30;
pub const STEEL_HEADBUTT_BONUS: i32 = 10;

pub fn steel_headbutt_damage(heads: bool) -> i32 {
    if heads { STEEL_HEADBUTT_BASE + STEEL_HEADBUTT_BONUS } else { STEEL_HEADBUTT_BASE }
}
