use godot::{
    classes::{
        AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Input, InputEvent, Node, Node2D,
        character_body_2d::MotionMode,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{VarDictionary, Variant, Vector2},
        },
    },
    obj::{Base, Singleton, WithBaseField},
    register::{
        GodotClass, godot_api,
        info::{PropertyInfo, PropertyUsageFlags},
    },
};

use crate::{
    RpgDirection,
    avatar::{character_sprite::CharacterSprite2D, follower::FollowerSet},
};

mod interact;

mod character_sprite;

mod follower;

#[derive(GodotClass)]
#[class(init, base = CharacterBody2D, rename = RpgCharacter2D)]
pub struct RpgCharacter2d {
    base: Base<CharacterBody2D>,

    /// The direction in which this character is facing.
    #[export]
    #[var(pub, set)]
    facing_dir: RpgDirection,

    /// Movement speed, in units(?) per second.
    #[export]
    #[export_group(name = "Movement")]
    #[var]
    #[init(val = 128.0)]
    move_speed: f32,

    /// Whether movement acceleration is enabled.
    #[export]
    #[export_subgroup(name = "Acceleration")]
    #[var]
    #[init(val = false)]
    acceleration_enabled: bool,

    /// The number of physics frames before movement accelerates.
    #[export]
    #[var]
    #[init(val = 180)]
    acceleration_delay: u32,

    /// Accelerated movement speed, in units(?) per second.
    #[export]
    #[var]
    #[init(val = 192.0)]
    acceleration_speed: f32,

    /// The maximum distance from which the player can interact with things.
    #[export]
    #[export_group(name = "Interaction")]
    #[var]
    #[init(val = 24.0)]
    interact_distance: f32,

    /// The number of frames of delay between this character's position and the positions of its followers.
    ///
    /// Note: changes only apply to followers added after the change.
    #[export]
    #[export_group(name = "Followers")]
    #[var]
    #[init(val = 12)]
    follower_delay: u32,

    /// whether an interact raycast is queued for the next physics process
    interact_queued: bool,

    sprite: Option<CharacterSprite2D>,

    /// The number of frames for which we've been moving without stopping
    acceleration_accumulator: u32,

    followers: FollowerSet,
}

impl RpgCharacter2d {
    /// The character's current movement speed, taking into account acceleration.
    pub const fn current_move_speed(&self) -> f32 {
        if self.acceleration_enabled && self.acceleration_accumulator >= self.acceleration_delay {
            self.acceleration_speed
        } else {
            self.move_speed
        }
    }
}

#[godot_api]
impl RpgCharacter2d {
    /// Emitted after interacting with something.
    #[signal]
    fn interacted_with(node: Gd<Node>, ray_intersection: VarDictionary);

    #[func]
    pub fn set_facing_dir(&mut self, dir: RpgDirection) {
        self.facing_dir = dir;
        if let Some(sprite) = self.sprite.as_mut() {
            sprite.set_dir(dir);
        }
    }

    #[func]
    pub fn facing_vec(&self) -> Vector2 {
        self.facing_dir.to_vector()
    }

    #[func]
    pub fn push_follower(&mut self, follower: Gd<Node2D>) {
        self.followers.push_follower(
            follower,
            // this should basically never actually fail to convert because i don't imagine anyone
            // is using this library on any platform with pointers smaller than 32 bits, but, you
            // know, just in case...
            usize::try_from(self.follower_delay).unwrap_or(12),
            1.0,
            follower::FollowerFrame {
                position: self.base().get_global_position(),
                facing: self.facing_dir,
            },
        );
    }
}

#[godot_api]
impl ICharacterBody2D for RpgCharacter2d {
    fn enter_tree(&mut self) {
        if !self.base().is_in_group("avatar") {
            self.base_mut().add_to_group("avatar");
        }
        self.base_mut().set_motion_mode(MotionMode::FLOATING);
    }

    fn on_validate_property(&self, info: &mut PropertyInfo) {
        if info.property_name == "motion_mode" {
            info.usage |= PropertyUsageFlags::READ_ONLY;
        }
    }

    fn ready(&mut self) {
        // TODO :: sprite

        if let Some(sprite) = self
            .base()
            .get_children()
            .iter_shared()
            .find_map(|child| child.try_cast::<AnimatedSprite2D>().ok())
        {
            let sprite = CharacterSprite2D::new(sprite, self.facing_dir);
            self.sprite = Some(sprite);
        }
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        let Some(mut vp) = self.base().get_viewport() else {
            return;
        };
        if event.is_action_pressed("toggle_menu") {
            tracing::error!("todo: toggle menu");
            vp.set_input_as_handled();
        } else if event.is_action_pressed("interact") {
            self.interact_queued = true;
            vp.set_input_as_handled();
        }
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

        let old_pos = self.base().get_global_position();

        // update velocity & sprite
        if godot::global::is_zero_approx(input_vec.length() as f64) {
            // we're not trying to move
            self.base_mut().set_velocity(Vector2::ZERO);
            // update acceleration accumulator
            self.acceleration_accumulator = 0;
            // update sprite
            if let Some(sprite) = self.sprite.as_mut() {
                sprite.ensure_stopped();
            }
        } else {
            // we're trying to move
            self.facing_dir = RpgDirection::from_vec(input_vec);
            let new_vel = self.facing_vec() * self.current_move_speed();
            self.base_mut().set_velocity(new_vel);
            // update acceleration accumulator
            // // i don't think anyone will ever actually move for 2^32 straight frames, but, you know,
            // // might as well account for the pannenkoeks of the world
            self.acceleration_accumulator = self.acceleration_accumulator.saturating_add(1);
            // update sprite
            if let Some(sprite) = self.sprite.as_mut() {
                sprite.set_dir(self.facing_dir);
                sprite.ensure_playing();
            }
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
                // if the thing we collided with has an on_collision function, call that
                if collider.has_method(ON_COLLISION_FN) {
                    collider.call(
                        ON_COLLISION_FN,
                        &[Variant::from(self.to_gd()), Variant::from(collision)],
                    );
                }
            }
        }

        // update followers
        let new_pos = self.base().get_global_position();
        let mv_vec = new_pos - old_pos;
        if mv_vec.length_squared()
            < Vector2::new(
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            )
            .length_squared()
        {
            // we didn't actually move (even if we tried), so let's have everybody take a break
            self.followers.stop();
        } else {
            // everybody keeps going
            self.followers.push_frame(new_pos, self.facing_dir);
        }
    }
}
