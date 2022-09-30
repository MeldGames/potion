use std::{
    f32::consts::PI,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use sabi::stage::NetworkSimulationAppExt;

use crate::attach::Attach;

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct DepositBox;

/// Component determining the value of specific items
/// as well as the global money of the players.
#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Value(u64);

impl Value {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn enough(&self, value: &Self) -> bool {
        self.0 >= value.0
    }

    pub fn set(&mut self, value: u64) {
        self.0 = value;
    }

    pub fn clear(&mut self) {
        self.set(0);
    }
}

impl Add<Value> for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl AddAssign<Value> for Value {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Sub<Value> for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.sub(rhs.0))
    }
}

impl SubAssign<Value> for Value {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub(rhs);
    }
}

pub struct DepositPlugin;
impl Plugin for DepositPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Value>();

        //app.add_system(display_money);

        app.add_network_system(deposit);
    }
}

pub fn deposit(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    deposits: Query<(Entity, &DepositBox)>,
    mut player_money: ResMut<Value>,
    mut values: Query<&mut Value>,
) {
    for (deposit, _) in &deposits {
        for (collider1, collider2, intersecting) in rapier_context.intersections_with(deposit) {
            let potential_sellable = if collider1 == deposit {
                collider2
            } else {
                collider1
            };

            if intersecting {
                if let Ok(mut value) = values.get_mut(potential_sellable) {
                    let sellable = potential_sellable;
                    *player_money += *value;

                    // we clear the value so we don't double sell this item.
                    value.clear();

                    commands.entity(sellable).despawn_recursive();
                }
            }
        }
    }
}

pub fn display_money(mut egui_context: ResMut<bevy_egui::EguiContext>, value: Res<Value>) {
    egui::Window::new("Value")
        .min_width(0.0)
        .default_width(1.0)
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(format!("Tendies: {:?}", value.get()));
        });
}

pub fn spawn_deposit_box(
    commands: &mut Commands,
    asset_server: &AssetServer,
    _meshes: &mut Assets<Mesh>,
    position: Transform,
) -> Entity {
    let crate_model = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/crate.gltf#Scene0"),
            ..default()
        })
        .insert(Name::new("Deposit Box Model"))
        .id();

    let lid_model = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/crate_lid.gltf#Scene0"),
            ..default()
        })
        .insert(Name::new("Lid Model"))
        .id();

    let deposit = commands
        .spawn_bundle(TransformBundle::from_transform(position))
        .insert_bundle((
            ColliderMassProperties::Density(50.0),
            RigidBody::Dynamic,
            Collider::cuboid(0.7, 0.55, 0.55),
            Name::new("Crate"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .insert(crate::DecompLoad(
            "assets/models/crate_decomp.obj".to_owned(),
        ))
        .insert_bundle(VisibilityBundle::default())
        .add_child(crate_model)
        .id();

    let lid_hinge = RevoluteJointBuilder::new(Vec3::X)
        .local_anchor1(Vec3::new(0.0, 1.6, -0.73))
        .limits([0.0, PI / 1.04]);
    let mut lid_hinge = lid_hinge.build();
    lid_hinge.set_contacts_enabled(false);

    let _lid = commands
        .spawn_bundle(TransformBundle::from_transform(position))
        .insert_bundle((
            ColliderMassProperties::Density(10.0),
            RigidBody::Dynamic,
            Collider::cuboid(0.7, 0.55, 0.55),
            Name::new("Lid"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .insert(crate::DecompLoad(
            "assets/models/crate_lid_decomp.obj".to_owned(),
        ))
        .insert_bundle(VisibilityBundle::default())
        .insert(ImpulseJoint::new(deposit, lid_hinge))
        .add_child(lid_model)
        .id();

    commands
        .spawn_bundle(TransformBundle::from_transform(position))
        .insert_bundle(Attach::all(deposit))
        .insert_bundle((Name::new("Deposit Area"), crate::physics::TERRAIN_GROUPING))
        .with_children(|children| {
            children
                .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, 0.175, 0.0,
                )))
                .insert(Collider::cuboid(0.475, 0.05, 0.25))
                .insert(DepositBox)
                .insert(Sensor);
        });

    deposit
}
