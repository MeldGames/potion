use bevy::prelude::Vec3;
use bevy_rapier3d::prelude::*;

fn main() {
    let vec = Vec3::new(1.0, 1.0, 0.0);
    dbg!(vec.normalize());
}
