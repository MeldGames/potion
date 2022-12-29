use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_rapier3d::prelude::*;

pub type HookData<'a> = &'a ContactFilter;

#[derive(Default, Debug, Clone, Component)]
pub struct ContactFilter(pub HashSet<Entity>);

pub struct ContactFilterHook;

impl<'a> PhysicsHooksWithQuery<HookData<'a>> for ContactFilterHook {
    fn modify_solver_contacts(
        &self,
        context: ContactModificationContextView,
        user_data: &Query<HookData>,
    ) {
        let mut should_clear = false;
        if let Ok(filter) = user_data.get(context.collider1()) {
            should_clear |= filter.0.contains(&context.collider2());
        }

        if let Ok(filter) = user_data.get(context.collider2()) {
            should_clear |= filter.0.contains(&context.collider1());
        }

        if should_clear {
            context.raw.solver_contacts.clear();
        }
    }
}
