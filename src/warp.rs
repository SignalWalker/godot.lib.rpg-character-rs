use std::cell::RefCell;

use godot::{
    classes::{
        Area2D, IArea2D, Node, Node2D, PackedScene,
        class_macros::private::virtuals::{Xrvrs::Gd, ZipReader::NodePath},
    },
    obj::{Base, WithBaseField, WithUserSignals},
    register::{GodotClass, godot_api},
};

#[cfg(feature = "scene-warp")]
mod scene_warp;
#[cfg(feature = "scene-warp")]
pub use scene_warp::*;

use crate::RpgCharacter2d;

#[derive(GodotClass)]
#[class(init, base = Area2D, rename = Warp2D)]
pub struct Warp2d {
    base: Base<Area2D>,

    /// The node to which to warp.
    #[export]
    #[var]
    target: NodePath,

    /// A scene transition node.
    #[export]
    #[var(pub, set)]
    transition: Option<Gd<PackedScene>>,

    transition_node_cache: RefCell<Option<Gd<Node>>>,
}

#[godot_api]
impl Warp2d {
    #[signal]
    fn warped(node: Gd<Node2D>, to: Gd<Node2D>);

    #[func]
    pub fn get_target_node(&self) -> Option<Gd<Node2D>> {
        self.base().try_get_node_as(&self.target)
    }

    #[func]
    pub fn set_transition(&mut self, transition: Option<Gd<PackedScene>>) {
        self.transition = transition;
        *self.transition_node_cache.borrow_mut() = None;
    }
}

#[godot_api]
impl IArea2D for Warp2d {
    fn enter_tree(&mut self) {
        let _ = self
            .signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }
}

impl Warp2d {
    pub fn get_transition_node(&self) -> Option<Gd<Node>> {
        if let Some(scene) = self.transition.as_ref() {
            if self.transition_node_cache.borrow().is_none() {
                let Some(node) = scene.instantiate() else {
                    tracing::error!(transition = %scene, "could not instantiate warp transition");
                    return None;
                };
                *self.transition_node_cache.borrow_mut() = Some(node);
            }
            self.transition_node_cache.borrow().clone()
        } else {
            None
        }
    }

    fn on_body_entered(&mut self, node: Gd<Node2D>) {
        let Ok(mut av) = node.try_cast::<RpgCharacter2d>() else {
            return;
        };

        let Some(target) = self.get_target_node() else {
            tracing::error!(target = %self.target, "could not get target node");
            return;
        };

        if let Some(_transition) = self.get_transition_node() {
            tracing::warn!("not yet implemented: warp transition animation");
            // TODO :: transition animation
        }

        av.set_global_position(target.get_global_position());
        self.signals().warped().emit(&av, &target);
    }
}
