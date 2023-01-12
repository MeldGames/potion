use std::fmt::Debug;

use bevy::ecs::{
    entity::Entities,
    query::{ReadOnlyWorldQuery, WorldQuery},
};

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_debug_lines::DebugLines;

use bevy_mod_wanderlust::Spring;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{JointAxis, MotorModel};

use crate::physics::{Muscle, GRAB_GROUPING, REST_GROUPING};

use super::controller::{ConnectedEntities, LookTransform};
use super::input::PlayerInput;
use super::prelude::*;
use crate::cauldron::NamedEntity;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrabJoint;

/// Entities currently grabbed onto.
#[derive(Deref, DerefMut, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct GrabbedEntities {
    pub grabbed: HashSet<Entity>,
}

#[derive(Deref, DerefMut, Default, Debug, Component, Clone, Reflect)]
#[reflect(Component)]
pub struct JointChildren(pub Vec<Entity>);

pub fn joint_children(
    mut commands: Commands,
    entities: &Entities,
    mut children: Query<&mut JointChildren>,
    joints: Query<(Entity, &ImpulseJoint), Without<GrabJoint>>,
) {
    for (entity, joint) in &joints {
        match children.get_mut(joint.parent) {
            Ok(mut children) => {
                if !children.contains(&entity) {
                    children.push(entity);
                }
            }
            _ => {
                if entities.contains(joint.parent) {
                    commands
                        .entity(joint.parent)
                        .insert(JointChildren(vec![entity]));
                }
            }
        }
    }
}

pub fn twist_arms(
    name: Query<&Name>,
    rapier_context: Res<RapierContext>,
    globals: Query<&GlobalTransform>,
    mut hands: Query<
        (
            Entity,
            &Grabbing,
            &GlobalTransform,
            Option<&Children>,
            &ConnectedEntities,
            &ConnectedMass,
            &mut GrabbedEntities,
        ),
        With<Hand>,
    >,
    grab_joints: Query<(&ImpulseJoint, &GrabJoint)>,
) {
}

pub fn grab_collider(
    mut commands: Commands,
    name: Query<&Name>,
    rapier_context: Res<RapierContext>,
    globals: Query<&GlobalTransform>,
    mut hands: Query<
        (
            Entity,
            &Grabbing,
            &GlobalTransform,
            Option<&Children>,
            &ConnectedEntities,
            &ConnectedMass,
            &mut GrabbedEntities,
        ),
        With<Hand>,
    >,
    grab_joints: Query<(&ImpulseJoint, &GrabJoint)>,
) {
    for (hand, grabbing, global, children, connected, mass, mut grabbed) in &mut hands {
        if grabbing.0 {
            let mut already_grabbing = false;

            if let Some(children) = children {
                for child in children.iter() {
                    if grab_joints.contains(*child) {
                        // We are already grabbing something so just skip this hand.
                        already_grabbing = true;
                        break;
                    }
                }
            }

            if already_grabbing {
                continue;
            }

            for contact_pair in rapier_context.contacts_with(hand) {
                let other_collider = if contact_pair.collider1() == hand {
                    contact_pair.collider2()
                } else {
                    contact_pair.collider1()
                };

                if connected.contains(&other_collider) {
                    continue;
                }

                let contact_points = contact_pair
                    .manifolds()
                    .map(|manifold| {
                        manifold
                            .solver_contacts()
                            .map(|contact| contact.point())
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                if contact_points.len() == 0 {
                    continue;
                }

                let mut closest_point = Vec3::ZERO;
                let mut closest_distance = f32::MAX;
                for point in &contact_points {
                    let dist = point.distance(global.translation());
                    if dist < closest_distance {
                        closest_point = *point;
                        closest_distance = dist;
                    }
                }

                if let Ok(other_global) = globals.get(other_collider) {
                    // convert back to local space.
                    let other_transform = other_global.compute_transform();
                    let other_matrix = other_global.compute_matrix();
                    let anchor1 = other_matrix.inverse().project_point3(closest_point)
                        * other_transform.scale;
                    let transform = global.compute_transform();
                    let matrix = global.compute_matrix();
                    let anchor2 = matrix.inverse().project_point3(closest_point) * transform.scale;

                    if let Ok(name) = name.get(other_collider) {
                        info!("grabbing {:?}", name.as_str());
                    } else {
                        info!("grabbing entity {:?}", other_collider);
                    }

                    let motor_model = MotorModel::ForceBased;
                    let max_force = 1000.0;
                    let stiffness = 20.0;
                    let damping = 0.4;
                    let mut grab_joint = SphericalJointBuilder::new()
                        .local_anchor1(anchor1)
                        .local_anchor2(anchor2)
                        .motor_model(JointAxis::AngX, motor_model)
                        .motor_model(JointAxis::AngY, motor_model)
                        .motor_model(JointAxis::AngZ, motor_model)
                        .motor_max_force(JointAxis::AngX, max_force)
                        .motor_max_force(JointAxis::AngY, max_force)
                        .motor_max_force(JointAxis::AngZ, max_force)
                        .motor_position(JointAxis::AngX, 0.0, stiffness, damping)
                        .motor_position(JointAxis::AngZ, 0.0, stiffness, damping)
                        .motor_position(JointAxis::AngY, 0.0, stiffness, damping)
                        .build();
                    grab_joint.set_contacts_enabled(false);

                    commands.entity(hand).add_children(|children| {
                        children
                            .spawn(ImpulseJoint::new(other_collider, grab_joint))
                            .insert(GrabJoint);
                    });

                    grabbed.insert(other_collider);

                    let extended = springy::rapier::ExtendedMass(mass.0);
                    //info!("extended mass: {:?}", extended);
                    //commands.entity(other_collider).insert(extended);
                }
            }
        } else {
            // clean up joints if we aren't grabbing anymore
            if let Some(children) = children {
                for child in children.iter() {
                    if let Ok((impulse_joint, _joint)) = grab_joints.get(*child) {
                        commands
                            .entity(impulse_joint.parent)
                            .remove::<springy::rapier::ExtendedMass>();
                        commands.entity(*child).despawn_recursive();
                        grabbed.remove(&*child);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct Grabbing(pub bool);

pub fn find_parent_with<'a, Q: WorldQuery, F: ReadOnlyWorldQuery>(
    query: &'a Query<Q, F>,
    parents: &'a Query<&Parent>,
    joints: &'a Query<&ImpulseJoint>,
    base: Entity,
) -> Option<<<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>> {
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

        if let Ok(joint) = joints.get(possible) {
            possibilities.push(joint.parent);
        }
    }

    queried
}

pub fn tense_arms(
    hands: Query<(Entity, &Grabbing), With<Hand>>,
    mut muscles: Query<(Entity, &mut Muscle)>,
    parents: Query<&Parent>,
    joints: Query<&ImpulseJoint>,
    names: Query<&Name>,
) {
    for (hand_entity, grabbing) in &hands {
        let mut entity = hand_entity;
        while let Ok((muscle_entity, mut muscle)) = muscles.get_mut(entity) {
            muscle.tense = grabbing.0;

            if let Ok(joint) = joints.get(entity) {
                entity = joint.parent;
            } else {
                break;
            }
        }
    }
}

pub fn player_grabby_hands(
    globals: Query<&GlobalTransform>,
    mut transforms: Query<&mut Transform>,
    inputs: Query<(
        &GlobalTransform,
        &LookTransform,
        &PlayerInput,
        &PlayerCamera,
        &Velocity,
    )>,
    ik_base: Query<&IKBase>,
    parents: Query<&Parent>,
    joints: Query<&ImpulseJoint>,
    ctx: Res<RapierContext>,
    mut hands: Query<(Entity, &mut Grabbing, &mut CollisionGroups, &ArmId, &MuscleIKTarget), With<Hand>>,
    names: Query<&Name>,
    mut lines: ResMut<DebugLines>,
) {
    let dt = ctx.integration_parameters.dt;

    for (hand_entity, mut grabbing, mut collision_groups, arm_id, muscle_ik_target) in &mut hands {
        let input = find_parent_with(&inputs, &parents, &joints, hand_entity);

        let (global, look, input, cam, velocity) = if let Some(input) = input {
            input
        } else {
            warn!("couldn't find parent input for hand entity");
            continue;
        };

        if input.grabby_hands(arm_id.0) {
            if let Ok(mut target_position) = transforms.get_mut(muscle_ik_target.0) {
                if let Ok(base) = ik_base.get(muscle_ik_target.0) {
                    if let Ok([shoulder, camera]) = globals.get_many([base.0, cam.0]) {
                        let direction = (camera.translation() - shoulder.translation()).normalize_or_zero();
                        target_position.translation = shoulder.translation() - direction * 1.5;
                    }
                }
            }

            grabbing.0 = true;
            *collision_groups = GRAB_GROUPING;
        } else {
            grabbing.0 = false;
            *collision_groups = REST_GROUPING;
        }
    }
}
