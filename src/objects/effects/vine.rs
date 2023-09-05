use super::EffectVelocity;
use crate::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_rapier3d::parry::{math::Isometry, query::PointQuery, shape::TypedShape};

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
    const DEBUG_TIME: f32 = 1000.0;

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

        let points = sample_points(&*ctx, global.translation(), 500, vine_range);
        info!("points: {:?}", points.len());
        let groups = crate::objects::group_points(points, |center, ray| {
            let alignment = center.normal.dot(ray.normal);

            let center_y = center.normal;
            let (center_x, center_z) = center_y.any_orthonormal_pair();
            let center_x = -center_x;
            let center_z = center_z;

            let offset = ray.point - center.point;
            let y = offset.dot(center_y);
            let x = offset.dot(center_x);
            let z = offset.dot(center_z);
            alignment >= 0.8 && y.abs() <= vine_radius && x.abs() <= vine_radius && z.abs() <= vine_radius
        });
        let colors = crate::objects::debug_colors(groups.len());
        info!("groups: {:?}", groups.len());

        for (index, group) in groups.iter().enumerate() {
            let center_y = group.center.normal;
            let (center_x, center_z) = center_y.any_orthonormal_pair();
            let center_x = -center_x;
            let center_z = center_z;

            let color = colors[index % colors.len()];
            for ray in &group.points {
                gizmos.sphere(DEBUG_TIME, ray.point, Quat::IDENTITY, 0.05, color);
            }

            let vine_align = center_x.normalize_or_zero();

            gizmos.ray(DEBUG_TIME, group.center.point, center_y * 0.5, Color::BLUE);
            gizmos.ray(DEBUG_TIME, group.center.point, vine_align * 0.5, Color::RED);

            let default_size = 0.1;
            let mut min = Vec3::splat(-default_size / 2.0);
            let mut max = Vec3::splat(default_size / 2.0);

            for ray in &group.points {
                let offset = ray.point - group.center.point;
                let aligned = Vec3::new(center_z.dot(offset), center_y.dot(offset), center_x.dot(offset));
                min = min.min(aligned);
                max = max.max(aligned);
            }

            gizmos.ray(DEBUG_TIME, group.center.point, center_y, Color::BLUE);
            gizmos.ray(DEBUG_TIME, group.center.point, center_x, Color::RED);
            gizmos.ray(DEBUG_TIME, group.center.point, center_z, Color::GREEN);

            // local space center

            let aabb = Aabb::from_min_max(min, max);
            let mut transform = Transform {
                translation: group.center.point, // + Vec3::from(aabb.center),
                scale: Vec3::from(aabb.half_extents * 2.0),
                ..default()
            }
            .looking_to(vine_align, center_y);

            let offset = transform.rotation * aabb.center;
            transform.translation -= Vec3::from(offset);
            gizmos.cuboid(DEBUG_TIME, transform, Color::CYAN);
            //gizmos.sphere(DEBUG_TIME, center.point - Vec3::from(aabb.center), Quat::IDENTITY, 0.05, Color::CYAN);
            //gizmos.sphere(DEBUG_TIME, center.point, Quat::IDENTITY, 0.05, Color::RED);
            /*
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
            */
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
