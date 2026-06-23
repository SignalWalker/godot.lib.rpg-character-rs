use godot::{
    meta::GodotConvert,
    register::property::{Export, Var},
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, GodotConvert, Var, Export, Default,
)]
#[godot(via = u8)]
#[repr(u8)]
pub enum RpgDirection {
    #[default]
    East = 0,
    SouthEast = 1,
    South = 2,
    SouthWest = 3,
    West = 4,
    NorthWest = 5,
    North = 6,
    NorthEast = 7,
}
