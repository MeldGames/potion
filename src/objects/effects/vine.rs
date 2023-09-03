use super::EffectVelocity;
use crate::prelude::*;
use bevy_rapier3d::parry::{math::Isometry, query::PointQuery, shape::TypedShape};
use bevy::render::primitives::Aabb;

#[derive(Component)]
pub struct VineEffect;

#[derive(Component)]
pub struct Vine;

pub fn sunflower_effect(mut gizmos: Gizmos) {
    for point in super::sunflower_circle(500, 0.0) {
        let shifted = Vec3::Y;
        let point = shifted + Vec3::new(point.x, 0.0, point.y);
        gizmos.sphere(point, Quat::IDENTITY, 0.01, Color::ORANGE);
    }

    for point in super::spiral_sphere(500) {
        let shifted = Vec3::Y * 2.0 + Vec3::Z * 2.0;
        let point = shifted + point;
        gizmos.sphere(point, Quat::IDENTITY, 0.01, Color::ORANGE);
    }
}

/// Despawn a vine if it isn't in contact with anything
/// other than another vine.
pub fn vine_despawn(
    mut commands: Commands,
    ctx: Res<RapierContext>,
    vines: Query<(Entity, &GlobalTransform, &Collider), With<Vine>>,
) {
    for (entity, global, collider) in &vines {
        let manifolds = crate::physics::contact_manifolds(
            &*ctx,
            global.translation(),
            Quat::IDENTITY,
            collider,
            &QueryFilter::default().exclude_sensors(),
        );

        let mut despawn = true;
        for (contact_entity, _) in manifolds {
            if vines.contains(contact_entity) {
                continue;
            }

            despawn = false;
        }

        if despawn {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Vine potion effect
///
/// Goals:
/// - Grabs and joints dynamic bodies to other dynamic bodies
///   or to kinematic/fixed bodies.
/// - Travel upwards, away from gravity, if the slope is steep
///   enough.
/// - Burnable
pub fn vine_effect(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,

    ctx: Res<RapierContext>,
    vine_effect: Query<(), Or<(With<VineEffect>, With<Vine>)>>,
    potions: Query<(Entity, &GlobalTransform, Option<&EffectVelocity>), With<VineEffect>>,
    globals: Query<&GlobalTransform>,
    colliders: Query<&Collider>,
    mut gizmos: ResMut<RetainedGizmos>,
) {
    const DEBUG_TIME: f32 = 10.0;

    let material = materials.add(StandardMaterial {
        base_color: Color::DARK_GREEN,
        perceptual_roughness: 0.2,
        ..default()
    });

    for (effect_entity, global, velocity) in &potions {
        commands.entity(effect_entity).remove::<VineEffect>();
    }

    let dt = ctx.integration_parameters.dt;
    for (effect_entity, global, velocity) in &potions {
        let velocity = if let Some(velocity) = velocity {
            if velocity.linear.length_squared() == 0.0 {
                Vec3::NEG_Y
            } else {
                velocity.linear
            }
        } else {
            Vec3::NEG_Y
        };

        //let effect_radius = 3.0;
        let vine_range = 2.0;
        let vine_radius = 0.5;
        let vine_height = 0.10;
        let half_height = vine_height / 2.0;

        gizmos.sphere(
            DEBUG_TIME,
            global.translation(),
            Quat::IDENTITY,
            vine_range,
            Color::PURPLE,
        );

        // Sample in a sphere around the impact point.
        let mut groups = Vec::new();
        let samples = sample_points(&*ctx, global.translation(), 300, vine_range);
        let mut to_sort = samples.clone();

        while let Some((center_entity, center)) = to_sort.pop() {
            let mut group_entities = Vec::new();
            let mut group_points = Vec::new();
            group_entities.push(center_entity);
            group_points.push(center);

            let mut to_remove = Vec::new();
            for (index, (other_entity, other_ray)) in to_sort.iter().enumerate() {
                let offset = center.point - other_ray.point;

                // accumulate points within this vine
                let height = center.normal.dot(offset).abs();
                let (x, z) = center.normal.any_orthonormal_pair();
                let x_diff = x.dot(offset).abs();
                let z_diff = z.dot(offset).abs();

                let within_bounds = height <= vine_radius && x_diff <= vine_radius && z_diff <= vine_radius;
                let aligned = center.normal.dot(other_ray.normal) >= 0.8;
                if within_bounds && aligned {
                    group_entities.push(*other_entity);
                    group_points.push(*other_ray);
                    to_remove.push(index);
                }
            }

            to_remove.sort_by(|a, b| b.cmp(a)); // descending
            for index in to_remove {
                to_sort.swap_remove(index);
            }

            groups.push((group_entities, center, group_points));
        }

        for (entity, ray) in &samples {
            //gizmos.sphere(DEBUG_TIME, ray.point, Quat::IDENTITY, 0.05, Color::RED);
            //gizmos.ray(10.0, ray.point, ray.normal * 0.2, Color::ORANGE);
        }

        for (entities, center, rays) in &groups {
            let mut sum = rays.iter().map(|ray| ray.normal).sum::<Vec3>();
            let normal = sum / rays.len() as f32;

            if normal.length_squared() <= 0.01 {
                continue;
            }

            /*
            gizmos.sphere(DEBUG_TIME, center.point, Quat::IDENTITY, 0.05, Color::RED);
            gizmos.sphere(
                DEBUG_TIME,
                center.point,
                Quat::IDENTITY,
                vine_radius,
                Color::ORANGE,
            );
            gizmos.ray(DEBUG_TIME, center.point, normal * 0.5, Color::BLUE);
            */

            let (x, z) = normal.any_orthonormal_pair();
            let vine_align = x.normalize_or_zero();

            gizmos.ray(DEBUG_TIME, center.point, normal * 0.5, Color::BLUE);
            gizmos.ray(DEBUG_TIME, center.point, vine_align * 0.5, Color::RED);

            let default_size = 0.1;
            let mut min = Vec3::splat(-default_size / 2.0);
            let mut max = Vec3::splat(default_size / 2.0);
            for ray in rays {
                let offset = ray.point - center.point;
                let aligned = Vec3::new(z.dot(offset), normal.dot(offset), x.dot(offset));
                min = min.min(aligned);
                max = max.max(aligned);
            }

            let aabb = Aabb::from_min_max(min, max);
            let transform = Transform {
                translation: center.point - Vec3::from(aabb.center),
                scale: Vec3::from(aabb.half_extents * 2.0),
                ..default()
            }
            .looking_to(vine_align, normal);
            gizmos.sphere(DEBUG_TIME, center.point - Vec3::from(aabb.center), Quat::IDENTITY, 0.05, Color::CYAN);
            gizmos.sphere(DEBUG_TIME, center.point, Quat::IDENTITY, 0.05, Color::RED);
            //gizmos.cuboid(DEBUG_TIME, transform, Color::CYAN);
            commands
                .spawn(SpatialBundle {
                    transform: transform,
                    ..default()
                })
                .insert(Vine)
                //.insert(VineEffect)
                .insert(material.clone())
                //.insert(RigidBody::Fixed)
                .insert(Sensor)
                .insert(ColliderBundle::collider(Collider::cuboid(0.5, 0.5, 0.5)));
        }
    }
}

pub fn sample_points(
    ctx: &RapierContext,
    from: Vec3,
    samples: usize,
    max_toi: f32,
) -> Vec<(Entity, RayIntersection)> {
    let mut results = Vec::new();

    for dir in super::spiral_sphere(samples) {
        let Some(result) = ctx.cast_ray_and_get_normal(
            from,
            dir,
            max_toi,
            false,
            QueryFilter::default().exclude_sensors(),
            //QueryFilter::default().exclude_sensors().exclude_dynamic(),
        ) else {
            continue;
        };

        results.push(result);
    }

    results
}
