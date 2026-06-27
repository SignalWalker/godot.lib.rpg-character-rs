use godot::{
    classes::{
        Node, PhysicsDirectSpaceState2D, PhysicsRayQueryParameters2D, World2D,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{Dictionary, Variant, array},
        },
    },
    obj::{WithBaseField, WithUserSignals},
};

use crate::InteractHandler;

#[derive(Debug, thiserror::Error)]
pub(super) enum InteractError {
    #[error("could not find world")]
    MissingWorld,
    #[error("world ({0}) does not have physics space state")]
    MissingSpaceState(Gd<World2D>),
    #[error("could not create ray query parameters in world {0} with space state {1}")]
    QueryCreation(Gd<World2D>, Gd<PhysicsDirectSpaceState2D>),
}

impl super::RpgCharacter2d {
    fn interact_with(&mut self, target: &mut Gd<Node>, raycast_data: Dictionary<Variant, Variant>) {
        let mut interacted = false;
        // for every InteractHandler held by the interactee...
        for handler in target
            .get_children()
            .iter_shared()
            .filter_map(|child| child.try_cast::<InteractHandler>().ok())
        {
            // check whether we can interact with it...
            if InteractHandler::can_interact(handler.clone(), self.to_gd(), raycast_data.clone()) {
                // and then interact with it
                interacted = true;
                InteractHandler::on_interact(handler.clone(), self.to_gd(), raycast_data.clone());
            }
        }
        if interacted {
            // ...and then tell everyone that we interacted with something
            self.signals()
                .interacted_with()
                .emit(&*target, &raycast_data.into_read_only());
        }
    }

    /// # Safety
    ///
    /// Must only be called during physics processing.
    pub(super) unsafe fn interact(&mut self) -> Result<(), InteractError> {
        let Some(world) = self.base().get_world_2d() else {
            return Err(InteractError::MissingWorld);
        };

        let Some(mut space) = world.get_direct_space_state() else {
            return Err(InteractError::MissingSpaceState(world));
        };

        // make the query parameters
        let self_pos = self.base().get_global_position();
        let trg_pos = self_pos + (self.facing_vec() * self.interact_distance);
        let Some(mut query) = PhysicsRayQueryParameters2D::create_ex(self_pos, trg_pos)
            // 0xFFFFFFFF collides with all layers
            .collision_mask(0xFFFFFFFF)
            // exclude ourselves from collision with the ray
            .exclude(&array![self.base().get_rid()])
            .done()
        else {
            return Err(InteractError::QueryCreation(world, space));
        };

        // don't just collide with shapes
        query.set_collide_with_areas(true);

        let result = space.intersect_ray(&query);
        if result.is_empty() {
            return Ok(());
        }

        tracing::trace!(%result, "interacting...");

        let mut target = result.at("collider").to::<Gd<Node>>();
        self.interact_with(&mut target, result);

        Ok(())
    }
}
