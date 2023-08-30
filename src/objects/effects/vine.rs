use super::EffectVelocity;
use crate::prelude::*;

#[derive(Component)]
pub struct VineEffect;

pub fn sunflower_effect(mut gizmos: Gizmos) {
    for point in sunflower(500, 0.0) {
        let point = Vec3::new(point.x, 0.0, point.y);
        gizmos.sphere(Vec3::Y + point, Quat::IDENTITY, 0.01, Color::ORANGE);
    }
}

pub fn vine_effect(
    ctx: Res<RapierContext>,
    potions: Query<(&GlobalTransform, Option<&EffectVelocity>), With<VineEffect>>,
    mut gizmos: ResMut<RetainedGizmos>,
) {
    let dt = ctx.integration_parameters.dt;
    for (global, velocity) in &potions {
        let default = Vec3::NEG_Y;
        let velocity = if let Some(velocity) = velocity {
            if velocity.linear.length_squared() == 0.0 {
                default
            } else {
                velocity.linear
            }
        } else {
            default
        };

        let dir = velocity.normalize_or_zero();

        let from = global.translation() + -dir * 1.5;
        gizmos.sphere(dt * 3.0, from, Quat::IDENTITY, 0.2, Color::BLUE);
        gizmos.ray(dt * 3.0, from, dir * 4.0, Color::BLUE);

        let points = sample_points(&*ctx, from, dir, 0.5, 4.0);

        for (entity, ray) in points {
            gizmos.ray(dt * 3.0, ray.point, ray.normal * 0.3, Color::YELLOW);
        }
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
    for sample in sunflower(100, 0.0) {
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
