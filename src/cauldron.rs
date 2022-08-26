use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_rapier3d::prelude::*;
use sabi::stage::NetworkSimulationAppExt;

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
