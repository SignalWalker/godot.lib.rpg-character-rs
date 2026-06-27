use godot::register::{GodotClass, godot_api};

use super::RpgDirection;

#[derive(GodotClass, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[class(no_init, rename = RpgDirection)]
struct RpgDirectionGd {}

#[godot_api]
impl RpgDirectionGd {
    #[constant]
    const EAST: RpgDirection = RpgDirection::East;
    #[constant]
    const SOUTH_EAST: RpgDirection = RpgDirection::SouthEast;
    #[constant]
    const SOUTH: RpgDirection = RpgDirection::South;
    #[constant]
    const SOUTH_WEST: RpgDirection = RpgDirection::SouthWest;
    #[constant]
    const WEST: RpgDirection = RpgDirection::West;
    #[constant]
    const NORTH_WEST: RpgDirection = RpgDirection::NorthWest;
    #[constant]
    const NORTH: RpgDirection = RpgDirection::North;
    #[constant]
    const NORTH_EAST: RpgDirection = RpgDirection::NorthEast;
}
