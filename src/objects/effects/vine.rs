use super::EffectVelocity;
use crate::prelude::*;

#[derive(Component)]
pub struct VineEffect;

#[derive(Component)]
pub struct Vine;

pub fn sunflower_effect(mut gizmos: Gizmos) {
    for point in sunflower(500, 0.0) {
        let point = Vec3::new(point.x, 0.0, point.y);
        gizmos.sphere(Vec3::Y + point, Quat::IDENTITY, 0.01, Color::ORANGE);
    }
}

pub fn vine_effect(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ctx: Res<RapierContext>,
    vine_effect: Query<(), Or<(With<VineEffect>, With<Vine>)>>,
    potions: Query<(Entity, &GlobalTransform, Option<&EffectVelocity>), With<VineEffect>>,
    mut gizmos: ResMut<RetainedGizmos>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::DARK_GREEN,
        perceptual_roughness: 0.2,
        ..default()
    });

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

        let dir = velocity.normalize_or_zero();

        let from = global.translation() + -dir * 1.5;
        gizmos.sphere(dt * 3.0, from, Quat::IDENTITY, 0.2, Color::BLUE);
        gizmos.ray(dt * 3.0, from, dir * 4.0, Color::BLUE);

        let points = sample_points(&*ctx, from, dir, 0.5, 4.0);

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
                .insert(material.clone())
                .insert(ColliderBundle::collider(Collider::cylinder(
                    0.25, 0.5,
                )));
            gizmos.ray(8.0, ray.point, ray.normal * 3.8, Color::PURPLE);
        }

        commands.entity(effect_entity).despawn_recursive();
    }
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
    for sample in sunflower(10, 0.0) {
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

/*
def sunflower(n, alpha=0, geodesic=False):
    points = []
    angle_stride = 360 * phi if geodesic else 2 * pi / phi ** 2
    b = round(alpha * sqrt(n))  # number of boundary points
    for k in range(1, n + 1):
        r = radius(k, n, b)
        theta = k * angle_stride
        points.append((r * cos(theta), r * sin(theta)))
    return points
*/

pub fn sunflower(n: usize, alpha: f32) -> Vec<Vec2> {
    let PHI: f32 = (1.0 + 5.0f32.sqrt()) / 2.0;
    let mut points = Vec::new();

    let angle_stride = 360.0 * PHI;
    let boundary_points = (alpha * (n as f32).sqrt());
    for k in 1..(n + 1) {
        let r = boundary_radius(k as f32, n as f32, boundary_points);
        let theta = k as f32 * angle_stride;
        points.push(Vec2::new(r * theta.cos(), r * theta.sin()));
    }

    points
}

pub fn boundary_radius(k: f32, n: f32, b: f32) -> f32 {
    if k > n - b {
        // put on the boundary
        1.0
    } else {
        // apply square root
        (k - 1.0 / 2.0).sqrt() / (n - (b + 1.0) / 2.0).sqrt()
    }
}
/*
function r = radius(k,n,b)
    if k>n-b
        r = 1;            % put on the boundary
    else
        r = sqrt(k-1/2)/sqrt(n-(b+1)/2);     % apply square root
    end
end
*/
