use godot::{
    classes::{
        Node,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{Dictionary, Variant},
        },
    },
    obj::{Base, WithUserSignals},
    register::{GodotClass, godot_api},
};

use crate::RpgCharacter2d;

#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct InteractHandler {
    base: Base<Node>,
}

#[godot_api]
impl InteractHandler {
    #[signal]
    fn interacted(avatar: Gd<RpgCharacter2d>, raycast_data: Dictionary<Variant, Variant>);

    #[func(gd_self, virtual)]
    pub fn can_interact(
        handler: Gd<InteractHandler>,
        avatar: Gd<RpgCharacter2d>,
        raycast_data: Dictionary<Variant, Variant>,
    ) -> bool {
        handler.bind().default_can_interact(avatar, raycast_data)
    }

    #[func]
    fn default_can_interact(
        &self,
        _av: Gd<RpgCharacter2d>,
        _raycast_data: Dictionary<Variant, Variant>,
    ) -> bool {
        true
    }

    #[func(gd_self, virtual)]
    #[allow(unused_mut, reason = "false positive")]
    pub fn on_interact(
        mut handler: Gd<Self>,
        av: Gd<RpgCharacter2d>,
        raycast_data: Dictionary<Variant, Variant>,
    ) {
        handler.bind_mut().default_on_interact(av, raycast_data)
    }

    #[func]
    #[inline]
    pub fn default_on_interact(
        &mut self,
        av: Gd<RpgCharacter2d>,
        raycast_data: Dictionary<Variant, Variant>,
    ) {
        self.signals().interacted().emit(&av, &raycast_data);
    }
}
