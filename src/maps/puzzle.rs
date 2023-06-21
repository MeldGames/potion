
use std::f32::consts::PI;

use crate::{
    attach::Attach,
    objects::{
        cauldron::Ingredient,
        store::{SecurityCheck, StoreItem},
    },
    physics::slot::{Slot, SlotGracePeriod, SlotSettings, Slottable},
    player::grab::{AimPrimitive, AutoAim},
};

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use bevy_rapier3d::prelude::*;

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(super::base_test::SetupPlugin);
        app.add_startup_system(setup);
    }
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

}