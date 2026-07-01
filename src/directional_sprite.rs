use godot::{
    classes::{AnimatedSprite2D, IAnimatedSprite2D, notify::CanvasItemNotification},
    obj::{Base, WithBaseField as _, WithUserSignals},
    register::{GodotClass, godot_api},
};
use godot_utils::DropHandle;

use crate::{RollingValue, RpgDirection, SpriteDirections};

// mod walk_sprite;
// pub use walk_sprite::*;

#[derive(GodotClass)]
#[class(tool, init, base = AnimatedSprite2D)]
pub struct DirectionalSprite2D {
    base: Base<AnimatedSprite2D>,

    #[export]
    #[var(set)]
    direction: RollingValue<RpgDirection>,

    supported_dirs: SpriteDirections,

    _frames_changed_handle: DropHandle,
}

#[godot_api]
impl IAnimatedSprite2D for DirectionalSprite2D {
    fn enter_tree(&mut self) {
        self.connect_signals();
        self.update_frames();
    }

    fn on_notification(&mut self, notif: CanvasItemNotification) {
        if notif == CanvasItemNotification::EXTENSION_RELOADED {
            self.connect_signals();
        }
    }
}

#[godot_api]
impl DirectionalSprite2D {
    #[func]
    pub fn set_direction(&mut self, dir: RpgDirection) {
        self.direction.push(dir);
        self.supported_dirs
            .apply_rolling_direction(self.to_gd(), self.direction);
    }
}

impl DirectionalSprite2D {
    pub const fn direction(&self) -> RollingValue<RpgDirection> {
        self.direction
    }

    fn connect_signals(&mut self) {
        self._frames_changed_handle = self
            .signals()
            .sprite_frames_changed()
            .connect_self(Self::update_frames)
            .into();
    }

    fn update_frames(&mut self) {
        let old_dirs = self.supported_dirs;
        let new_dirs = self.base().get_sprite_frames().as_ref().map_or(
            SpriteDirections::None,
            SpriteDirections::check_sprite_frames,
        );
        if old_dirs == new_dirs {
            return;
        }
        self.supported_dirs = new_dirs;
        self.supported_dirs
            .apply_rolling_direction(self.to_gd(), self.direction);
    }
}
