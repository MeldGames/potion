use bevy::prelude::Vec3;
use bevy_rapier3d::prelude::*;

fn main() {
    let collider = Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5);
    let serialized = bincode::serialize(&collider.raw);
    if let Ok(serialized) = serialized {
        dbg!(serialized.len());
    }
}
