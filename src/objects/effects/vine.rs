use super::EffectVelocity;
use crate::prelude::*;
use crate::objects::shape_closest_point;

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

        let effect_radius = 3.0;

        let manifolds = crate::physics::contact_manifolds(
            &*ctx,
            global.translation(),
            Quat::IDENTITY,
            &Collider::ball(effect_radius),
            &QueryFilter::default().exclude_sensors(),
        );
        gizmos.sphere(
            2.0,
            global.translation(),
            Quat::IDENTITY,
            effect_radius,
            Color::CYAN,
        );

        let contacting = manifolds
            .into_iter()
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        for c1 in &contacting {
            let c1 = *c1;
            let Ok(c1_collider) = colliders.get(c1) else {
                continue;
            };
            let c1_global = globals.get(c1).unwrap_or(&GlobalTransform::IDENTITY);

            let point = shape_closest_point(c1_global, &*c1_collider, c1_global.translation());
            info!("closest: {:?}", point);
            gizmos.sphere(1000.0, point, Quat::IDENTITY, 0.05, Color::RED);

            for c2 in &contacting {
                let c2 = *c2;
                if c1 == c2 {
                    continue;
                }
            }
        }
        /*
        for (entity, manifold) in &manifolds {
            let contact_global = globals.get(*entity).unwrap_or(&GlobalTransform::IDENTITY);

            for point in &manifold.points {
                let point = contact_global.transform_point(point.local_p2.into());
                //let normal = contact_global.transform_point(manifold.local_n2.into());
                let normal = manifold.local_n2.into();
                gizmos.sphere(1000.0, point, Quat::IDENTITY, 0.05, Color::RED);
                gizmos.ray(1000.0, point, normal, Color::RED);

                let (x, z) = normal.any_orthonormal_pair();
                commands
                    .spawn(SpatialBundle {
                        transform: Transform {
                            translation: point,
                            ..default()
                        }
                        .looking_to(x, normal),
                        ..default()
                    })
                    .insert(Vine)
                    //.insert(VineEffect)
                    .insert(material.clone())
                    //.insert(RigidBody::Fixed)
                    .insert(Sensor)
                    .insert(ColliderBundle::collider(Collider::cylinder(0.25, 0.5)));
            }
        }
        */
        /*
            commands
                .spawn(SpatialBundle {
                    transform: Transform {
                        translation: ray.point,
                        ..default()
                    }
                    .looking_to(z, ray.normal),
                    ..default()
                })
                .insert(Vine)
                //.insert(VineEffect)
                .insert(material.clone())
                .insert(RigidBody::Fixed)
                .insert(ColliderBundle::collider(Collider::cylinder(0.25, 0.5)));
            //gizmos.ray(8.0, ray.point, ray.normal * 3.8, Color::PURPLE);
        }
        */

        /*
        for (entity, ray) in sample_points(&*ctx, global.translation() + Vec3::Y, 100, 3.0) {
            gizmos.sphere(1000.0, ray.point, Quat::IDENTITY, 0.05, Color::RED);
            gizmos.ray(1000.0, ray.point, ray.normal * 0.2, Color::ORANGE);
        }
        */
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
        ) else {
            continue;
        };

        results.push(result);
    }

    results
}
