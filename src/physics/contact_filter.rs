use crate::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::utils::HashSet;

/// Filter specific entities when solving contacts.
#[derive(Default, Debug, Clone, Component)]
pub struct ContactFilter(pub HashSet<Entity>);

/// Don't try to solve any contacts on this entity.
#[derive(Default, Debug, Clone, Component)]
pub struct NoContacts;

#[derive(SystemParam)]
pub struct ContactFilterHook<'w, 's> {
    filters: Query<'w, 's, &'static ContactFilter>,
    no_contacts: Query<'w, 's, &'static NoContacts>,
}

impl<'w, 's> BevyPhysicsHooks for ContactFilterHook<'w, 's> {
    fn modify_solver_contacts(&self, context: ContactModificationContextView) {
        let should_clear = || -> bool {
            if self.no_contacts.contains(context.collider1())
                || self.no_contacts.contains(context.collider2())
            {
                return true;
            }

            if let Ok(filter) = self.filters.get(context.collider1()) {
                if filter.0.contains(&context.collider2()) {
                    return true;
                }
            }

            if let Ok(filter) = self.filters.get(context.collider2()) {
                if filter.0.contains(&context.collider1()) {
                    return true;
                }
            }

            false
        }();

        if should_clear {
            context.raw.solver_contacts.clear();
        }
    }
}
