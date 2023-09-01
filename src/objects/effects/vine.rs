use super::EffectVelocity;
use crate::prelude::*;

#[derive(Component)]
pub struct VineEffect;

#[derive(Component)]
pub struct Vine;

pub fn sunflower_effect(mut gizmos: Gizmos) {
    for point in super::sunflower(500, 0.0) {
        let point = Vec3::new(point.x, 0.0, point.y);
        gizmos.sphere(Vec3::Y + point, Quat::IDENTITY, 0.01, Color::ORANGE);
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
    can_delete: Query<(), With<RapierRigidBodyHandle>>,

    ctx: Res<RapierContext>,
    vine_effect: Query<(), Or<(With<VineEffect>, With<Vine>)>>,
    potions: Query<(Entity, &GlobalTransform, Option<&EffectVelocity>), With<VineEffect>>,
    mut gizmos: ResMut<RetainedGizmos>,

    mut increment: Local<usize>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::DARK_GREEN,
        perceptual_roughness: 0.2,
        ..default()
    });

    let per_step = 3;
    let count = potions.iter().count().max(per_step).max(1) / per_step;
    *increment = *increment % count;

    //info!("increment: {:?}", increment);

    for (effect_entity, global, velocity) in &potions {
        if can_delete.contains(effect_entity) {
            commands.entity(effect_entity).remove::<VineEffect>();
        }
    }

    let dt = ctx.integration_parameters.dt;
    for (effect_entity, global, velocity) in
        potions.iter().skip(*increment * per_step).take(per_step)
    {
        let velocity = if let Some(velocity) = velocity {
            if velocity.linear.length_squared() == 0.0 {
                Vec3::NEG_Y
            } else {
                velocity.linear
            }
        } else {
            Vec3::NEG_Y
        };

        let dir = velocity.normalize_or_zero();

        let from = global.translation() + -dir * 1.5;
        //gizmos.sphere(dt * 3.0, from, Quat::IDENTITY, 0.2, Color::BLUE);
        //gizmos.ray(dt * 3.0, from, dir * 4.0, Color::BLUE);

        let points = sample_points(&*ctx, from, dir, 1.0, 4.0);

        for (entity, ray) in points {
            if vine_effect.contains(entity) {
                continue;
            }

            let (x, z) = ray.normal.any_orthonormal_pair();
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
    }

    *increment += 1;
}

pub fn sample_points(
    ctx: &RapierContext,
    from: Vec3,
    dir: Vec3,
    radius: f32,
    max_toi: f32,
) -> Vec<(Entity, RayIntersection)> {
    let mut results = Vec::new();
    let dir = dir.normalize_or_zero();
    if dir.length_squared() == 0.0 {
        return results;
    }

    let diameter = radius * 2.0;
    for sample in super::sunflower(10, 0.0) {
        let sample = Vec3::new(sample.x, 0.0, sample.y) * diameter;

        let Some(result) = ctx.cast_ray_and_get_normal(
            from + sample,
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
