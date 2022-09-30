use ::egui::Ui;
use bevy::{ecs::query::WorldQuery, prelude::*, utils::HashSet};
use bevy_inspector_egui::{Inspectable, InspectableRegistry, RegisterInspectable};
use bevy_rapier3d::prelude::*;
use sabi::stage::NetworkSimulationAppExt;

use crate::{attach::Attach, ColliderLoad};

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
        app.register_type::<Soup>();
        app.register_type::<HashSet<Entity>>();
        info!("registering soup");
        app.register_inspectable_raw::<Soup, _>(|soup, ui, context| -> bool {
            let mut vec = soup.ingredients.iter().cloned().collect::<Vec<_>>();
            vec.as_mut_slice().ui(ui, Default::default(), context)
        });
        app.register_inspectable_raw::<HashSet<Entity>, _>(|soup, ui, context| -> bool {
            let mut vec = soup.iter().cloned().collect::<Vec<_>>();
            vec.as_mut_slice().ui(ui, Default::default(), context)
        });
        app.add_network_system(insert_ingredient);
    }
}

#[derive(Default, Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Soup {
    pub ingredients: HashSet<Entity>,
}

impl Soup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, ingredient: Entity) -> bool {
        self.ingredients.insert(ingredient)
    }

    pub fn remove(&mut self, ingredient: Entity) -> bool {
        self.ingredients.remove(&ingredient)
    }
}

pub fn insert_ingredient(
    name: Query<&Name>,
    mut soups: Query<(Entity, &mut Soup)>,
    mut collision_events: EventReader<CollisionEvent>,
    ingredients: Query<(Entity, &Ingredient)>,
) {
    for collision_event in collision_events.iter() {
        let ((soup_entity, mut soup), (ingredient_entity, ingredient), colliding) =
            match collision_event {
                &CollisionEvent::Started(collider1, collider2, flags) => {
                    let (soup, potential) = if let Ok(soup) = soups.get_mut(collider1) {
                        (soup, collider2)
                    } else if let Ok(soup) = soups.get_mut(collider2) {
                        (soup, collider1)
                    } else {
                        continue;
                    };

                    if let Ok(ingredient) = ingredients.get(potential) {
                        (soup, ingredient, true)
                    } else {
                        continue;
                    }
                }
                &CollisionEvent::Stopped(collider1, collider2, flags) => {
                    let (soup, potential) = if let Ok(soup) = soups.get_mut(collider1) {
                        (soup, collider2)
                    } else if let Ok(soup) = soups.get_mut(collider2) {
                        (soup, collider1)
                    } else {
                        continue;
                    };

                    if let Ok(ingredient) = ingredients.get(potential) {
                        (soup, ingredient, false)
                    } else {
                        continue;
                    }
                }
            };

        if colliding {
            if soup.insert(ingredient_entity) {
                info!(
                    "inserted {:?} into soup {:?}",
                    name.named(ingredient_entity),
                    name.named(soup_entity),
                );
            }
        } else {
            if soup.remove(ingredient_entity) {
                info!(
                    "removed {:?} from soup {:?}",
                    name.named(ingredient_entity),
                    name.named(soup_entity),
                );
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
            ColliderMassProperties::Density(100.0),
            /*
                       ColliderMassProperties::MassProperties(MassProperties {
                           local_center_of_mass: Vec3::new(0.0, -0.2, 0.0),
                           mass: 15.0,
                           principal_inertia: Vec3::ONE,
                           principal_inertia_local_frame: Quat::IDENTITY,
                       }),
            */
            ReadMassProperties::default(),
            RigidBody::Dynamic,
            Velocity::default(),
            ExternalImpulse::default(),
            crate::store::StoreItem,
            Collider::cylinder(0.4, 0.75),
            Name::new("Cauldron"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .insert(crate::DecompLoad(
            "assets/models/cauldron_decomp.obj".to_owned(),
        ))
        .insert(level_collision_mesh)
        .id();

    commands
        .spawn_bundle(TransformBundle::from_transform(position))
        .insert_bundle(Attach::all(cauldron))
        .insert_bundle((
            Name::new("Cauldron Deposit"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .with_children(|children| {
            children
                .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, 0.25, 0.0,
                )))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collider::cylinder(0.4, 0.55))
                .insert(Cauldron)
                .insert(Soup::default())
                .insert(Sensor);
        });

    cauldron
}
