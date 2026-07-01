use godot::{
    classes::{AnimatedSprite2D, SpriteFrames, class_macros::private::virtuals::Xrvrs::Gd},
    obj::Inherits,
};

use crate::{RollingValue, RpgDirection};

pub const NORTH_ANIM_NAME: &str = "walk_north";
pub const SOUTH_ANIM_NAME: &str = "walk_south";
pub const EAST_ANIM_NAME: &str = "walk_east";
pub const WEST_ANIM_NAME: &str = "walk_west";
pub const NORTHEAST_ANIM_NAME: &str = "walk_northeast";
pub const NORTHWEST_ANIM_NAME: &str = "walk_northwest";
pub const SOUTHEAST_ANIM_NAME: &str = "walk_southeast";
pub const SOUTHWEST_ANIM_NAME: &str = "walk_southwest";

/// The directions supported by a sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SpriteDirections {
    #[default]
    None,
    /// Cardinals (N, S, E, W)
    Cardinals,
    /// Cardinals & intercardinals (N, S, E, W, NE, SE, NW, SW)
    All,
}

impl SpriteDirections {
    pub const fn get_anim_name_for(dir: RpgDirection) -> &'static str {
        match dir {
            RpgDirection::East => EAST_ANIM_NAME,
            RpgDirection::SouthEast => SOUTHEAST_ANIM_NAME,
            RpgDirection::South => SOUTH_ANIM_NAME,
            RpgDirection::SouthWest => SOUTHWEST_ANIM_NAME,
            RpgDirection::West => WEST_ANIM_NAME,
            RpgDirection::NorthWest => NORTHWEST_ANIM_NAME,
            RpgDirection::North => NORTH_ANIM_NAME,
            RpgDirection::NorthEast => NORTHEAST_ANIM_NAME,
        }
    }

    /// Get the nearest supported direction to the given direction, biased towards the "sticky" direction.
    pub const fn nearest_to(self, sticky: RpgDirection, dir: RpgDirection) -> RpgDirection {
        match self {
            Self::None => sticky,
            Self::All => dir,
            Self::Cardinals => dir.nearest_cardinal(sticky),
        }
    }

    pub const fn nearest_anim_to(self, sticky: RpgDirection, dir: RpgDirection) -> &'static str {
        Self::get_anim_name_for(self.nearest_to(sticky, dir))
    }

    /// Get the directions supported by the given sprite.
    pub fn check_sprite(sprite: &Gd<AnimatedSprite2D>) -> Self {
        let Some(frames) = sprite.get_sprite_frames() else {
            return Self::None;
        };
        Self::check_sprite_frames(&frames)
    }

    /// Get the directions supported by the given sprite frames.
    pub fn check_sprite_frames(frames: &Gd<SpriteFrames>) -> Self {
        if frames.has_animation(NORTH_ANIM_NAME)
            && frames.has_animation(SOUTH_ANIM_NAME)
            && frames.has_animation(EAST_ANIM_NAME)
            && frames.has_animation(WEST_ANIM_NAME)
        {
            if frames.has_animation(NORTHWEST_ANIM_NAME)
                && frames.has_animation(NORTHEAST_ANIM_NAME)
                && frames.has_animation(SOUTHWEST_ANIM_NAME)
                && frames.has_animation(SOUTHEAST_ANIM_NAME)
            {
                SpriteDirections::All
            } else {
                SpriteDirections::Cardinals
            }
        } else {
            SpriteDirections::None
        }
    }

    pub fn apply_direction(
        self,
        sprite: Gd<impl Inherits<AnimatedSprite2D>>,
        prev_dir: RpgDirection,
        dir: RpgDirection,
    ) {
        let anim_name = self.nearest_anim_to(prev_dir, dir);
        let mut sprite = sprite.upcast();
        if sprite.is_playing() {
            let fr = sprite.get_frame();
            let pr = sprite.get_frame_progress();
            sprite.play_ex().name(anim_name).done();
            sprite.set_frame_and_progress(fr, pr);
        } else {
            sprite.set_animation(anim_name);
            sprite.set_frame(1);
        }
    }

    pub fn apply_rolling_direction(
        self,
        sprite: Gd<impl Inherits<AnimatedSprite2D>>,
        dir: RollingValue<RpgDirection>,
    ) {
        self.apply_direction(sprite, dir.prev, dir.current);
    }
}
