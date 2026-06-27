use godot::classes::{AnimatedSprite2D, class_macros::private::virtuals::Xrvrs::Gd};

use crate::RpgDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SpriteDirections {
    None,
    Cardinals,
    All,
}

impl SpriteDirections {
    const fn nearest_to(self, sticky: RpgDirection, dir: RpgDirection) -> RpgDirection {
        match self {
            Self::None => sticky,
            Self::All => dir,
            Self::Cardinals => dir.nearest_cardinal(sticky),
        }
    }
}

pub struct CharacterSprite2d {
    pub sprite: Gd<AnimatedSprite2D>,

    directions: SpriteDirections,

    anim_dir: RpgDirection,

    prev_fr_pr: Option<(i32, f32)>,
}

impl CharacterSprite2d {
    pub fn new(sprite: Gd<AnimatedSprite2D>, initial_dir: RpgDirection) -> Self {
        let directions = Self::check_animations(&sprite);
        let mut res = Self {
            directions,
            sprite,
            anim_dir: directions.nearest_to(RpgDirection::East, initial_dir),
            prev_fr_pr: None,
        };
        res.set_dir(initial_dir);
        res
    }

    const fn get_anim_name_for(dir: RpgDirection) -> &'static str {
        match dir {
            RpgDirection::East => "walk_east",
            RpgDirection::SouthEast => "walk_southeast",
            RpgDirection::South => "walk_south",
            RpgDirection::SouthWest => "walk_southwest",
            RpgDirection::West => "walk_west",
            RpgDirection::NorthWest => "walk_northwest",
            RpgDirection::North => "walk_north",
            RpgDirection::NorthEast => "walk_northeast",
        }
    }

    fn check_animations(sprite: &Gd<AnimatedSprite2D>) -> SpriteDirections {
        let Some(frames) = sprite.get_sprite_frames() else {
            return SpriteDirections::None;
        };
        if frames.has_animation("walk_north")
            && frames.has_animation("walk_south")
            && frames.has_animation("walk_east")
            && frames.has_animation("walk_west")
        {
            if frames.has_animation("walk_northwest")
                && frames.has_animation("walk_northeast")
                && frames.has_animation("walk_southwest")
                && frames.has_animation("walk_southeast")
            {
                SpriteDirections::All
            } else {
                SpriteDirections::Cardinals
            }
        } else {
            SpriteDirections::None
        }
    }

    pub fn set_dir(&mut self, dir: RpgDirection) {
        self.anim_dir = self.directions.nearest_to(self.anim_dir, dir);
        let anim_name = Self::get_anim_name_for(self.anim_dir);
        if self.sprite.is_playing() {
            let fr = self.sprite.get_frame();
            let pr = self.sprite.get_frame_progress();
            self.sprite.play_ex().name(anim_name).done();
            self.sprite.set_frame_and_progress(fr, pr);
        } else {
            self.sprite.set_animation(anim_name);
            self.sprite.set_frame(1);
        }
    }

    pub fn ensure_playing(&mut self) {
        if !self.sprite.is_playing() {
            self.sprite.play();
            if let Some((fr, pr)) = self.prev_fr_pr.take() {
                self.sprite.set_frame_and_progress(fr, pr);
            }
        }
    }

    pub fn ensure_stopped(&mut self) {
        if self.sprite.is_playing() {
            self.prev_fr_pr = Some((self.sprite.get_frame(), self.sprite.get_frame_progress()));
            self.sprite.stop();
            self.sprite.set_frame(1);
        }
    }
}
