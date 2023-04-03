use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_rapier3d::prelude::*;

#[derive(Default, Debug, Clone, Component)]
pub struct ContactFilter(pub HashSet<Entity>);

#[derive(SystemParam)]
pub struct ContactFilterHook<'w, 's> {
    filters: Query<'w, 's, &'static ContactFilter>,
}

impl<'w, 's> BevyPhysicsHooks for ContactFilterHook<'w, 's> {
    fn modify_solver_contacts(&self, context: ContactModificationContextView) {
        let mut should_clear = false;
        if let Ok(filter) = self.filters.get(context.collider1()) {
            should_clear |= filter.0.contains(&context.collider2());
        }

        if let Ok(filter) = self.filters.get(context.collider2()) {
            should_clear |= filter.0.contains(&context.collider1());
        }

        if should_clear {
            context.raw.solver_contacts.clear();
        }
    }
}
