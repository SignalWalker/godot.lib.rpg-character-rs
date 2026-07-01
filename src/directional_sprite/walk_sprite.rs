use godot::{
    classes::{
        AnimatedSprite2D, IAnimatedSprite2D, class_macros::private::virtuals::ZipReader::Vector2,
        notify::CanvasItemNotification,
    },
    init::is_editor_hint,
    obj::{Base, WithBaseField as _, WithUserSignals as _},
    register::{GodotClass, godot_api, property::PhantomVar},
};

use crate::{AnimatedSpriteExt, AnimationResumeData, RpgDirection};

#[derive(GodotClass)]
#[class(tool, init, base = AnimatedSprite2D)]
pub struct WalkSprite2D {
    base: Base<AnimatedSprite2D>,

    #[export]
    #[var(get = get_direction_gd, set)]
    direction: PhantomVar<RpgDirection>,

    prev_pos: Vector2,

    sprite_data: DirectionalSpriteData,
    resume_data: Option<AnimationResumeData>,
}

#[godot_api]
impl WalkSprite2D {
    #[func]
    pub fn set_direction(&mut self, dir: RpgDirection) {
        self.sprite_data.set_dir(self.to_gd().upcast_mut(), dir);
    }

    #[func(rename = get_direction)]
    fn get_direction_gd(&self) -> RpgDirection {
        self.direction()
    }
}

impl WalkSprite2D {
    pub const fn direction(&self) -> RpgDirection {
        self.sprite_data.direction()
    }

    fn connect_signals(&mut self) {
        let _ = self
            .signals()
            .sprite_frames_changed()
            .connect_self(Self::update_frames);
    }

    fn update_frames(&mut self) {
        self.sprite_data.set_sprite(self.to_gd().upcast_mut());
    }
}

#[godot_api]
impl IAnimatedSprite2D for WalkSprite2D {
    fn enter_tree(&mut self) {
        self.prev_pos = self.base().get_global_position();
        self.connect_signals();
        self.update_frames();
    }

    fn on_notification(&mut self, notif: CanvasItemNotification) {
        if notif == CanvasItemNotification::EXTENSION_RELOADED {
            self.connect_signals();
        }
    }

    fn physics_process(&mut self, _delta: f32) {
        if is_editor_hint() {
            return;
        }

        let new_pos = self.base().get_global_position();

        let pos_diff = new_pos - self.prev_pos;
        if pos_diff.length_squared() < 1.0 {
            // we didn't move
            if self.base().is_playing() {
                let resume_data = self.base_mut().stop_with_resume_data();
                self.base_mut().set_frame(1);
                self.resume_data = Some(resume_data);
            }
        } else {
            // we moved
            let new_dir = RpgDirection::from_vec(pos_diff);
            self.sprite_data.set_dir(self.to_gd().upcast_mut(), new_dir);
            if !self.base().is_playing() {
                if let Some(resume_data) = self.resume_data.take() {
                    self.base_mut().resume(resume_data);
                } else {
                    self.base_mut().play();
                }
            }
            self.prev_pos = new_pos;
        }
    }
}
