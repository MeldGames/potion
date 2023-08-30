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
