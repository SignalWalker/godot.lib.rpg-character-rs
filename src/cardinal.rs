use godot::{
    classes::class_macros::private::virtuals::ZipReader::Vector2,
    meta::GodotConvert,
    register::property::{Export, Var},
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, GodotConvert, Var, Export, Default,
)]
#[godot(via = u8)]
#[repr(u8)]
pub enum RpgDirection {
    East = 0,
    SouthEast = 1,
    #[default]
    South = 2,
    SouthWest = 3,
    West = 4,
    NorthWest = 5,
    North = 6,
    NorthEast = 7,
}

const DIR_VECTORS: [Vector2; 8] = {
    // lmao is this actually what that's called what a nerd
    use std::f32::consts::FRAC_1_SQRT_2 as MU;
    [
        Vector2::new(1.0, 0.0),  // east
        Vector2::new(MU, MU),    // southeast
        Vector2::new(0.0, 1.0),  // south
        Vector2::new(-MU, MU),   // southwest
        Vector2::new(-1.0, 0.0), // west
        Vector2::new(-MU, -MU),  // northwest
        Vector2::new(0.0, -1.0), // north
        Vector2::new(MU, -MU),   // northeast
    ]
};

impl RpgDirection {
    /// Return a unit vector in this direction.
    #[inline]
    pub const fn to_vector(self) -> Vector2 {
        DIR_VECTORS[self as usize]
    }

    fn angle_index_generic<const DIVISIONS: u32>(angle: f32) -> u8 {
        use std::f32::consts::{PI, TAU};
        let scl = (DIVISIONS - 1) as f32 / TAU;
        let angle = angle + PI;
        (((((angle * scl) + 1.0).round() as u32) % DIVISIONS) / 2) as u8
    }

    pub fn from_radians(angle: f32) -> Self {
        match Self::angle_index_generic::<16>(angle) {
            0 => Self::West,
            1 => Self::NorthWest,
            2 => Self::North,
            3 => Self::NorthEast,
            4 => Self::East,
            5 => Self::SouthEast,
            6 => Self::South,
            7 => Self::SouthWest,
            _ => unreachable!(
                "angle_index_generic::<16>() should never return a number outside of 0..8"
            ),
        }
    }

    pub fn from_radians_cardinal(angle: f32) -> Self {
        match Self::angle_index_generic::<8>(angle) {
            0 => Self::West,
            1 => Self::North,
            2 => Self::East,
            3 => Self::South,
            _ => unreachable!(
                "angle_index_generic::<8>() should never return a number outside of 0..4"
            ),
        }
    }

    pub fn from_vec(vec: Vector2) -> Self {
        Self::from_radians(vec.angle())
    }

    pub fn from_vec_cardinal(vec: Vector2) -> Self {
        Self::from_radians_cardinal(vec.angle())
    }

    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::East => "east",
            Self::SouthEast => "southeast",
            Self::South => "south",
            Self::SouthWest => "southwest",
            Self::West => "west",
            Self::NorthWest => "northwest",
            Self::North => "north",
            Self::NorthEast => "northeast",
        }
    }

    pub const fn is_cardinal(self) -> bool {
        matches!(
            self,
            RpgDirection::East | RpgDirection::South | RpgDirection::West | RpgDirection::North
        )
    }

    pub const fn nearest_cardinal(self, sticky: RpgDirection) -> Self {
        match self {
            Self::East | Self::South | Self::West | Self::North => self,
            dir => {
                let diff = (dir as u8).abs_diff(sticky as u8);
                if sticky.is_cardinal() && (diff <= 1 || diff == 7) {
                    sticky
                } else {
                    match self {
                        Self::SouthEast => Self::East,
                        Self::SouthWest => Self::South,
                        Self::NorthWest => Self::West,
                        Self::NorthEast => Self::North,
                        // this is unreachable (but the unreachable!() macro is not const)
                        _ => Self::East,
                    }
                }
            }
        }
    }
}
