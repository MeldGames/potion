use crate::prelude::*;

pub trait ContextExt {
    fn colliders(&self, entity: Entity) -> Vec<Entity>;
}

impl ContextExt for RapierContext {
    fn colliders(&self, entity: Entity) -> Vec<Entity> {
        let Some(body_handle) = self.entity2body().get(&entity) else { return Vec::new() };
        let Some(body) = self.bodies.get(*body_handle) else { return Vec::new() } ;

        body.colliders()
            .iter()
            .filter_map(|handle| self.collider_entity(*handle))
            .collect()
    }
}
