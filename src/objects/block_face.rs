use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum BlockFace {
    None = 0,
    Left = 1,
    Right = 2,
    Front = 3,
    Back = 4,
    Top = 5,
    Bottom = 6,
}