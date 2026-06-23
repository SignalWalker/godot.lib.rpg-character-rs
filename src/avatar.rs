use godot::{
    classes::{
        CharacterBody2D, ICharacterBody2D, Input, Node,
        character_body_2d::MotionMode,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{VarDictionary, Variant, Vector2},
        },
    },
    obj::{Base, Singleton, WithBaseField},
    register::{GodotClass, godot_api},
};

use crate::RpgDirection;

mod interact;

#[derive(GodotClass)]
#[class(init, base = CharacterBody2D, rename = RpgPlayer2D)]
pub struct RpgPlayer2d {
    base: Base<CharacterBody2D>,

    /// Movement speed, in units(?) per second.
    #[export]
    #[var]
    #[init(val = 128.0)]
    move_speed: f32,

    /// The initial direction in which to face.
    #[export]
    #[var(pub, set)]
    facing_dir: RpgDirection,

    /// The maximum distance from which the player can interact with things.
    #[export]
    #[var]
    #[init(val = 24.0)]
    interact_distance: f32,

    /// whether an interact raycast is queued for the next physics process
    interact_queued: bool,
}

#[godot_api]
impl RpgPlayer2d {
    /// Emitted after interacting with something.
    #[signal]
    fn interacted_with(node: Gd<Node>, ray_intersection: VarDictionary);

    #[func]
    pub fn set_facing_dir(&mut self, dir: RpgDirection) {
        todo!()
    }

    #[func]
    pub fn facing_vec(&self) -> Vector2 {
        todo!()
    }
}

#[godot_api]
impl ICharacterBody2D for RpgPlayer2d {
    fn enter_tree(&mut self) {
        if !self.base().is_in_group("avatar") {
            self.base_mut().add_to_group("avatar");
        }
        self.base_mut().set_motion_mode(MotionMode::FLOATING);
    }

    fn ready(&mut self) {
        // TODO :: sprite
    }

    fn physics_process(&mut self, _delta: f32) {
        const ON_COLLISION_FN: &str = "rpg_on_character_collision";

        if self.interact_queued {
            self.interact_queued = false;
            if let Err(error) = unsafe { self.interact() } {
                tracing::error!(rpg_character_2d = %self.to_gd(), %error, "interaction failed");
            }
        }

        let input_vec =
            Input::singleton().get_vector("move_left", "move_right", "move_up", "move_down");

        // update velocity
        if godot::global::is_zero_approx(input_vec.length() as f64) {
            // we're not moving
            self.base_mut().set_velocity(Vector2::ZERO);
            // TODO :: stop animation
        } else {
            // we're moving
            // TODO :: update facing
            let new_vel = self.facing_vec() * self.move_speed;
            self.base_mut().set_velocity(new_vel);
            // TODO :: walk animation
        }

        // move the character
        if self.base_mut().move_and_slide() {
            // handle collisions
            for collision in (0..self.base().get_slide_collision_count()).map(|index| {
                self.base()
                    .get_slide_collision(index)
                    .expect("this should only fail if we messed up the slide collision range")
            }) {
                let Some(mut collider) = collision
                    .get_collider()
                    .and_then(|col| col.try_cast::<Node>().ok())
                else {
                    continue;
                };
                // collide with everything
                if collider.has_method(ON_COLLISION_FN) {
                    collider.call(
                        ON_COLLISION_FN,
                        &[Variant::from(self.to_gd()), Variant::from(collision)],
                    );
                }
            }
        }
    }
}
