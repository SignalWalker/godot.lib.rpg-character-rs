use std::rc::Rc;

use futures::TryFutureExt as _;
use godot::{
    classes::{
        Area2D, IArea2D, Node, Node2D, PackedScene,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{GString, NodePath},
        },
        resource_loader::CacheMode,
    },
    obj::{Base, WithBaseField, WithUserSignals},
    register::{GodotClass, godot_api},
};
use scene_manager::{SceneManager, gd_api::SceneManagerNode};

use crate::RpgCharacter2d;

#[derive(GodotClass)]
#[class(init, base = Area2D, rename = SceneWarp2D)]
pub struct SceneWarp2d {
    base: Base<Area2D>,

    /// The scene to which to warp.
    #[export]
    #[var(
        hint = FILE,
        hint_string = "*.tscn"
    )]
    target_scene: GString,

    /// The path the the node, relative to the root of the target scene, to which to warp.
    #[export]
    #[var]
    target_path: GString,

    /// A scene transition.
    #[export]
    #[var]
    transition: Option<Gd<PackedScene>>,

    warping: bool,
}

#[godot_api]
impl SceneWarp2d {
    #[func]
    pub fn get_target_node(&self) -> Option<Gd<Node2D>> {
        self.base()
            .try_get_node_as(&NodePath::from(&self.target_path))
    }
}

#[godot_api]
impl IArea2D for SceneWarp2d {
    fn enter_tree(&mut self) {
        let _ = self
            .signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }
}

impl SceneWarp2d {
    fn on_body_entered(&mut self, node: Gd<Node2D>) {
        fn get_scene_manager(warp: &SceneWarp2d) -> Option<Rc<SceneManager>> {
            let Some(node) = warp.base().get_node_or_null("/root/SceneManager") else {
                tracing::error!(warp = %warp.to_gd(), "could not get SceneManager: node not found at /root/SceneManager");
                return None;
            };
            match node.try_cast::<SceneManagerNode>() {
                Ok(m) => Some(m.bind().manager().clone()),
                Err(node) => {
                    tracing::error!(warp = %warp.to_gd(), %node, "could not get SceneManager: node found at /root/SceneManager is not of type SceneManagerNode");
                    None
                }
            }
        }

        if self.warping {
            // i honestly don't know why, but, for some reason, areas seem to get double
            // body_entered signals, so we'll check for that and skip if it happens
            return;
        }

        let Ok(_av) = node.try_cast::<RpgCharacter2d>() else {
            // we only want to warp player characters
            return;
        };

        self.warping = true;

        let Some(scene_manager) = get_scene_manager(self) else {
            // we already emitted errors
            return;
        };

        if let Some(transition_scene) = self.transition.clone() {
            self.warping = true;
            self.warp_with_transition(transition_scene, scene_manager);
        } else {
            // load the target scene
            let target_scene = match scene_manager::resource::load_node_from_path(
                &self.target_scene,
                CacheMode::REUSE,
            ) {
                Ok(n) => n,
                Err(error) => {
                    tracing::error!(%error, "could not load target scene");
                    return;
                }
            };
            // defer scene swap until idle time
            self.run_deferred(move |warp: &mut Self| {
                // pop the old scene
                if let Some(mut old) = unsafe { scene_manager.pop_scene() } {
                    // we have to queue this here because otherwise we'd free the warp node right now
                    old.queue_free();
                }
                // push the target scene
                if let Err(error) = unsafe { scene_manager.push_scene(target_scene.clone()) } {
                    tracing::error!(%error, "could not push target scene");
                    return;
                };
                // find and reposition the player character
                Self::find_and_reposition_avatar(&target_scene, &warp.target_path);
            });
        }

        // av.set_global_position(target.get_global_position());
        // self.signals().warped().emit(&av, &target);
    }

    fn find_and_reposition_avatar(scene: &Gd<Node>, target_path: &GString) {
        // find the target node
        let Some(target) = scene.try_get_node_as::<Node2D>(&NodePath::from(target_path)) else {
            tracing::error!(path = %target_path, "could not find scene warp target node");
            return;
        };
        // find the player character
        let Some(mut av) = crate::find_first_avatar(scene.clone()) else {
            tracing::error!("warped to scene without avatar");
            return;
        };
        // reposition
        av.set_global_position(target.get_global_position());
    }

    fn warp_with_transition(
        &mut self,
        transition_scene: Gd<PackedScene>,
        scene_manager: Rc<SceneManager>,
    ) {
        // instantiate transition
        let Some(transition) = transition_scene.instantiate() else {
            tracing::error!(transition = %transition_scene, "could not instantiate scene transition");
            return;
        };
        // begin loading next scene
        let target_path = self.target_path.clone();
        let next_scene = match scene_manager::resource::load_threaded_to_node(
            self.target_scene.clone(),
            CacheMode::REUSE,
            false,
        ) {
            Ok(n) => n,
            Err(error) => {
                tracing::error!(%error, "could not start loading target scene");
                return;
            }
        }
        .inspect_ok(move |target_scene| {
            // find and reposition the player character in the new scene
            //
            // (we're doing this in the future task so that this happens while the player can't
            // see it)
            Self::find_and_reposition_avatar(target_scene, &target_path)
        });
        // defer transition start to idle time
        self.run_deferred(move |_: &mut Self| {
            // start the scene transition
            let transition_task =
                match unsafe { scene_manager.transition_scene(transition, next_scene) } {
                    Ok(t) => t,
                    Err(error) => {
                        tracing::error!(%error, "could not start scene transition");
                        return;
                    }
                };
            // run the scene transition
            godot::task::spawn(async move {
                if let Err(error) = transition_task.await {
                    tracing::error!(%error, "could not finish scene transition");
                }
            });
        });
    }
}
