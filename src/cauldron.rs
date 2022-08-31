use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_rapier3d::prelude::*;
use sabi::stage::NetworkSimulationAppExt;

use crate::{follow::Follow, ColliderLoad};

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Cauldron;

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Ingredient;

pub trait NamedEntity {
    fn named(&self, entity: Entity) -> String;
}

impl<'w, 's, F: WorldQuery> NamedEntity for Query<'w, 's, &Name, F> {
    fn named(&self, entity: Entity) -> String {
        match self.get_component::<Name>(entity) {
            Ok(name) => name.as_str().to_owned(),
            _ => format!("{:?}", entity),
        }
    }
}

pub struct CauldronPlugin;
impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.add_network_system(insert_ingredient);
    }
}

pub fn insert_ingredient(
    mut commands: Commands,
    name: Query<&Name>,
    rapier_context: Res<RapierContext>,
    cauldrons: Query<(Entity, Option<&Children>), With<Cauldron>>,
    ingredients: Query<&Ingredient>,
) {
    for (cauldron, children) in &cauldrons {
        for (collider1, collider2, intersecting) in rapier_context.intersections_with(cauldron) {
            let potential_ingredient = if collider1 == cauldron {
                collider2
            } else {
                collider1
            };

            if intersecting {
                if ingredients.contains(potential_ingredient) {
                    let already_added = children
                        .map(|children| children.iter().any(|child| *child == potential_ingredient))
                        .unwrap_or(false);
                    if !already_added {
                        let ingredient = potential_ingredient;
                        info!("adding {} to the cauldron", name.named(ingredient));
                        commands.entity(cauldron).add_child(ingredient);
                    }
                }
            }
        }
    }
}

pub fn spawn_cauldron(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Transform,
) -> Entity {
    let level_collision_mesh: Handle<Mesh> =
        asset_server.load("models/cauldron.glb#Mesh0/Primitive0");
    let cauldron = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/cauldron.glb#Scene0"),
            transform: position,
            ..default()
        })
        .insert_bundle((
            //ColliderMassProperties::Density(100.0),
            ColliderMassProperties::MassProperties(MassProperties {
                local_center_of_mass: Vec3::new(0.0, -0.6, 0.0),
                mass: 15.0,
                principal_inertia: Vec3::ONE,
                principal_inertia_local_frame: Quat::IDENTITY,
            }),
            ReadMassProperties::default(),
            RigidBody::Dynamic,
            Velocity::default(),
            ExternalImpulse::default(),
            crate::store::StoreItem,
            Collider::cylinder(0.4, 0.75),
            Name::new("Cauldron"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .insert(ColliderLoad)
        .insert(level_collision_mesh)
        .id();

    commands
        .spawn_bundle(TransformBundle::from_transform(position))
        .insert_bundle(Follow::all(cauldron))
        .insert_bundle((
            Name::new("Cauldron Deposit"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .with_children(|children| {
            children
                .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, 0.25, 0.0,
                )))
                .insert(Collider::cylinder(0.4, 0.55))
                .insert(Cauldron)
                .insert(Sensor);
        });

    cauldron
}
