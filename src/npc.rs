use godot::{
    classes::{
        AnimatedSprite2D, IStaticBody2D, Node, StaticBody2D,
        class_macros::private::virtuals::{Xrvrs::Gd, ZipReader::AnyDictionary},
        notify::CanvasItemNotification,
    },
    obj::{Base, WithBaseField as _, WithUserSignals},
    register::{GodotClass, godot_api},
};
use godot_utils::DropHandle;

use crate::{CharacterSprite2d, RpgCharacter2d, RpgDirection};

#[derive(GodotClass)]
#[class(tool, init, base = StaticBody2D, rename = StaticNpc2D)]
pub struct StaticNpc2d {
    base: Base<StaticBody2D>,

    #[export]
    #[var(pub, set)]
    facing_dir: RpgDirection,

    sprite: Option<CharacterSprite2d>,

    child_enter_handle: DropHandle,
    sprite_exit_handle: DropHandle,
}

impl StaticNpc2d {
    fn connect_signals(&mut self) {
        fn on_sprite_exiting(npc: &mut StaticNpc2d) {
            npc.child_enter_handle = npc
                .signals()
                .child_entered_tree()
                .connect_self(on_child_entered_tree)
                .into();
            npc.sprite_exit_handle = DropHandle::default();
            npc.sprite = None;
        }
        fn on_child_entered_tree(npc: &mut StaticNpc2d, child: Gd<Node>) {
            if npc.sprite.is_none()
                && let Ok(sprite) = child.try_cast::<AnimatedSprite2D>()
            {
                npc.sprite_exit_handle = sprite
                    .signals()
                    .tree_exiting()
                    .connect_other(npc, on_sprite_exiting)
                    .into();
                npc.child_enter_handle = DropHandle::default();
                npc.sprite = Some(CharacterSprite2d::new(sprite, npc.facing_dir));
            }
        }
        match self.sprite.as_ref() {
            None => {
                self.sprite_exit_handle = DropHandle::default();
                self.child_enter_handle = self
                    .signals()
                    .child_entered_tree()
                    .connect_self(on_child_entered_tree)
                    .into();
            }
            Some(sprite) => {
                self.child_enter_handle = DropHandle::default();
                self.sprite_exit_handle = sprite
                    .sprite
                    .signals()
                    .tree_exiting()
                    .connect_other(self, on_sprite_exiting)
                    .into();
            }
        }
    }
}

#[godot_api]
impl StaticNpc2d {
    #[func]
    pub fn set_facing_dir(&mut self, dir: RpgDirection) {
        self.facing_dir = dir;
        if let Some(sprite) = self.sprite.as_mut() {
            sprite.set_dir(dir);
        }
    }

    #[func(gd_self, virtual)]
    pub fn rpg_can_interact(
        npc: Gd<Self>,
        av: Gd<RpgCharacter2d>,
        raycast_data: AnyDictionary,
    ) -> bool {
        npc.bind().default_rpg_can_interact(av, raycast_data)
    }

    #[func]
    #[inline]
    pub fn default_rpg_can_interact(
        &self,
        _av: Gd<RpgCharacter2d>,
        _raycast_data: AnyDictionary,
    ) -> bool {
        true
    }

    #[func(gd_self, virtual)]
    #[allow(unused_mut, reason = "false positive")]
    pub fn rpg_on_interact(mut npc: Gd<Self>, av: Gd<RpgCharacter2d>, raycast_data: AnyDictionary) {
        npc.bind_mut().default_rpg_on_interact(av, raycast_data)
    }

    #[func]
    #[inline]
    pub fn default_rpg_on_interact(
        &mut self,
        av: Gd<RpgCharacter2d>,
        _raycast_data: AnyDictionary,
    ) {
        self.set_facing_dir(RpgDirection::from_vec(
            av.get_global_position() - self.base().get_global_position(),
        ));
    }
}

#[godot_api]
impl IStaticBody2D for StaticNpc2d {
    fn enter_tree(&mut self) {
        self.connect_signals();
    }

    fn on_notification(&mut self, notif: CanvasItemNotification) {
        // necessary because this is an @tool class and we rely on signals
        if notif == CanvasItemNotification::EXTENSION_RELOADED {
            self.connect_signals();
        }
    }
}
