use godot::{
    classes::{
        Node, PhysicsDirectSpaceState2D, PhysicsRayQueryParameters2D, World2D,
        class_macros::private::virtuals::{
            Xrvrs::Gd,
            ZipReader::{Variant, array},
        },
    },
    obj::{WithBaseField, WithUserSignals},
};

#[derive(Debug, thiserror::Error)]
pub(super) enum InteractError {
    #[error("could not find world")]
    MissingWorld,
    #[error("world ({0}) does not have physics space state")]
    MissingSpaceState(Gd<World2D>),
    #[error("could not create ray query parameters in world {0} with space state {1}")]
    QueryCreation(Gd<World2D>, Gd<PhysicsDirectSpaceState2D>),
}

impl super::RpgPlayer2d {
    /// # Safety
    ///
    /// Must only be called during physics processing.
    pub(super) unsafe fn interact(&mut self) -> Result<(), InteractError> {
        const CAN_INTERACT_FN: &str = "rpg_can_interact";
        const ON_INTERACT_FN: &str = "rpg_on_interact";

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

        let mut t = result.at("collider").to::<Gd<Node>>();
        // if we're allowed to interact with it (either it doesn't have can_interact() or it does
        // and it returns true)...
        if !t.has_method(CAN_INTERACT_FN)
            || t.call(
                CAN_INTERACT_FN,
                &[Variant::from(self.to_gd()), Variant::from(result.clone())],
            )
            .booleanize()
        {
            // ...do any special interactions...
            if t.has_method(ON_INTERACT_FN) {
                t.call(
                    ON_INTERACT_FN,
                    &[Variant::from(self.to_gd()), Variant::from(result.clone())],
                );
            }
            // ...and then tell everyone that we interacted with something
            self.signals()
                .interacted_with()
                .emit(&t, &result.into_read_only());
        }

        Ok(())
    }
}
