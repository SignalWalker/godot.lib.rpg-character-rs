use godot::{
    classes::{
        AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Input, Node,
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

use crate::RpgDirection;

#[derive(GodotClass)]
#[class(init, base = CharacterBody2D)]
pub struct RpgFollower2d {
    base: Base<CharacterBody2D>,
}
