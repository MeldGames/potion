use crate::prelude::*;
use bevy_rapier3d::na::Isometry3;
use bevy_rapier3d::parry::{
    bounding_volume::BoundingVolume,
    query::{DefaultQueryDispatcher, PersistentQueryDispatcher},
};
use bevy_rapier3d::rapier::geometry::ContactManifold;

pub trait ContextExt {
    fn colliders(&self, entity: Entity) -> Vec<Entity>;

    /// Get a list of contacts for a given shape.
    fn contact_manifolds(
        &self,
        position: Vec3,
        rotation: Quat,
        collider: &Collider,
        filter: &QueryFilter,
    ) -> Vec<(Entity, ContactManifold)>;

    fn correct_penetration(
        &self,
        position: Vec3,
        rotation: Quat,
        collider: &Collider,
        filter: &QueryFilter,
    ) -> Vec3;
}

impl ContextExt for RapierContext {
    fn colliders(&self, entity: Entity) -> Vec<Entity> {
        let Some(body_handle) = self.entity2body().get(&entity) else {
            return Vec::new();
        };
        let Some(body) = self.bodies.get(*body_handle) else {
            return Vec::new();
        };

        body.colliders()
            .iter()
            .filter_map(|handle| self.collider_entity(*handle))
            .collect()
    }

    fn contact_manifolds(
        &self,
        position: Vec3,
        rotation: Quat,
        collider: &Collider,
        filter: &QueryFilter,
    ) -> Vec<(Entity, ContactManifold)> {
        const FUDGE: f32 = 0.05;

        let physics_scale = self.physics_scale();

        let shape = &collider.raw;
        let shape_iso = Isometry3 {
            translation: (position / physics_scale).into(),
            rotation: rotation.into(),
        };

        let shape_aabb = shape.compute_aabb(&shape_iso).loosened(FUDGE);

        let mut manifolds = Vec::new();
        self.query_pipeline
            .colliders_with_aabb_intersecting_aabb(&shape_aabb, |handle| {
                if let Some(collider) = self.colliders.get(*handle) {
                    if RapierContext::with_query_filter(&self, *filter, |rapier_filter| {
                        rapier_filter.test(&self.bodies, *handle, collider)
                    }) {
                        let mut new_manifolds = Vec::new();
                        let pos12 = shape_iso.inv_mul(collider.position());
                        let _ = DefaultQueryDispatcher.contact_manifolds(
                            &pos12,
                            shape.as_ref(),
                            collider.shape(),
                            0.01,
                            &mut new_manifolds,
                            &mut None,
                        );

                        if let Some(entity) = self.collider_entity(*handle) {
                            manifolds.extend(
                                new_manifolds.into_iter().map(|manifold| (entity, manifold)),
                            );
                        }
                    }
                }

                true
            });

        manifolds
    }

    fn correct_penetration(
        &self,
        position: Vec3,
        rotation: Quat,
        collider: &Collider,
        filter: &QueryFilter,
    ) -> Vec3 {
        let mut push = Vec3::ZERO;
        for i in 0..10 {
            let manifolds = self.contact_manifolds(
                position + push,
                Quat::IDENTITY,
                collider,
                filter,
            );

            for (entity, manifold) in manifolds {
                let normal = Vec3::from(manifold.local_n1);
                if let Some(contact) = manifold.find_deepest_contact() {
                    if contact.dist.abs() < 0.01 {
                        continue;
                    }

                    push += normal * contact.dist;
                } else {
                    continue;
                }

                break;
            }
        }

        position + push
    }
}
