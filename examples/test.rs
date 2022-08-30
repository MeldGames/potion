use bevy::prelude::Vec3;

fn main() {
    let vec = Vec3::new(1.0, 1.0, 0.0);
    dbg!(vec.normalize());
}
