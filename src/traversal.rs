use crate::prelude::*;
use bevy::{
    ecs::{
        entity::Entities,
        query::{ReadOnlyQueryData, QueryFilter, WorldQuery, QueryData},
    },
    prelude::*,
    utils::HashSet,
};

pub struct HierarchyTraversalPlugin;

impl Plugin for HierarchyTraversalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<JointChildren>();

        app.add_systems(FixedUpdate, joint_children);
    }
}

pub mod prelude {
    pub use super::{find_children_with, find_parent_with, JointChildren};
}

pub fn find_parent_with<'a, Q: QueryData, F: QueryFilter>(
    query: &'a Query<Q, F>,
    parents: &'a Query<&Parent>,
    impulse: &'a Query<&ImpulseJoint>,
    base: Entity,
) -> Option<<<Q as QueryData>::ReadOnly as WorldQuery>::Item<'a>> {
    let mut checked = HashSet::new();
    let mut possibilities = vec![base];
    let mut queried = None;

    while let Some(possible) = possibilities.pop() {
        checked.insert(possible);

        queried = query.get(possible).ok();
        if queried.is_some() {
            break;
        }

        if let Ok(parent) = parents.get(possible) {
            possibilities.push(parent.get());
        }

        if let Ok(joint) = impulse.get(possible) {
            possibilities.push(joint.parent);
        }
    }

    queried
}

pub fn find_children_with<'a, Q: QueryData, F: QueryFilter>(
    query: &'a Query<Q, F>,
    children: &'a Query<&Children>,
    joint_children: &'a Query<&JointChildren>,
    base: Entity,
) -> Vec<<<Q as QueryData>::ReadOnly as WorldQuery>::Item<'a>> {
    let mut queried = Vec::new();
    let mut possibilities = vec![base];

    while let Some(possible) = possibilities.pop() {
        if let Ok(query) = query.get(possible) {
            queried.push(query);
        }

        if let Ok(children) = children.get(possible) {
            possibilities.extend(children.iter());
        }

        if let Ok(joint_children) = joint_children.get(possible) {
            possibilities.extend(joint_children.iter());
        }
    }

    queried
}

#[derive(Deref, DerefMut, Default, Debug, Component, Clone, Reflect)]
#[reflect(Component)]
pub struct JointChildren(pub Vec<Entity>);

pub fn joint_children(
    mut commands: Commands,
    entities: &Entities,
    mut children: Query<&mut JointChildren>,
    joints: Query<(Entity, &ImpulseJoint), Without<GrabJoint>>,
    multibody: Query<(Entity, &MultibodyJoint)>,
) {
    let pairs = joints
        .iter()
        .map(|(entity, joint)| (entity, joint.parent))
        .chain(
            multibody
                .iter()
                .map(|(entity, joint)| (entity, joint.parent)),
        );

    for (entity, parent) in pairs {
        match children.get_mut(parent) {
            Ok(mut children) => {
                if !children.contains(&entity) {
                    children.push(entity);
                }
            }
            _ => {
                if entities.contains(parent) {
                    commands.entity(parent).insert(JointChildren(vec![entity]));
                }
            }
        }
    }
}
