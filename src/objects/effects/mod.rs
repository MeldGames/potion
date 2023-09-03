use crate::prelude::*;

pub mod vine;

#[derive(Component)]
pub struct EffectVelocity {
    pub linear: Vec3,
}

pub struct EffectPlugin;
impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (vine::vine_effect, vine::vine_despawn));
        app.add_systems(Update, (vine::sunflower_effect));
    }
}

// helper methods

/// Uniform "sunflower seeding" sampling in a circle.
pub fn sunflower_circle(samples: usize, boundary_smoothing: f32) -> Vec<Vec2> {
    let n = samples;
    let alpha = boundary_smoothing;
    let phi: f32 = (1.0 + 5.0f32.sqrt()) / 2.0;
    let mut points = Vec::new();

    let angle_stride = 360.0 * phi;
    let boundary_points = (alpha * (n as f32).sqrt());
    for k in 1..(n + 1) {
        let r = boundary_radius(k as f32, n as f32, boundary_points);
        let theta = k as f32 * angle_stride;
        points.push(Vec2::new(r * theta.cos(), r * theta.sin()));
    }

    points
}

/// Uniform sampling in a sphere.
pub fn spiral_sphere(samples: usize) -> Vec<Vec3> {
    let n = samples;
    let phi: f32 = (1.0 + 5.0f32.sqrt()) / 2.0;

    let mut points = Vec::new();
    for i in 0..n {
        let i = i as f32;
        let y = 1.0 - (i / (n - 1) as f32) * 2.0;
        let radius = (1.0 - y * y).sqrt();
        let theta = phi * i;

        let x = theta.cos() * radius;
        let z = theta.sin() * radius;
        points.push(Vec3::new(x, y, z));
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

pub fn shape_closest_point(
    collider_global: &GlobalTransform,
    collider: &Collider,
    point: Vec3,
) -> Vec3 {
    use bevy_rapier3d::parry::{math::Isometry, query::PointQuery, shape::TypedShape};

    let (_, rotation, translation) = collider_global.to_scale_rotation_translation();
    let iso = Isometry {
        translation: translation.into(),
        rotation: rotation.into(),
    };

    let point_projection = match collider.raw.as_typed_shape() {
        TypedShape::Ball(ball) => ball.project_point(&iso, &point.into(), true),
        TypedShape::Cuboid(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::Cylinder(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::ConvexPolyhedron(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::TriMesh(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::Capsule(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::Compound(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::HalfSpace(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::Cone(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::HeightField(raw) => raw.project_point(&iso, &point.into(), true),
        _ => {
            unimplemented!("{:?}", collider.raw.shape_type());
        }
    };

    point_projection.point.into()
}
