use crate::prelude::*;

pub mod vine;

#[derive(Component)]
pub struct EffectVelocity {
    pub linear: Vec3,
}

pub struct EffectPlugin;
impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (vine::vine_effect));
        app.add_systems(Update, (vine::sunflower_effect));
    }
}

// helper methods

/// Uniform "sunflower seeding" sampling in a circle.
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
