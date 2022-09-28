use std::fmt::Debug;

use bevy::input::mouse::MouseWheel;
use bevy::utils::HashSet;
use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use std::f32::consts::PI;

use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_wanderlust::{
    CharacterControllerBundle, CharacterControllerPreset, ControllerInput, ControllerPhysicsBundle,
    ControllerSettings, ControllerState, Spring,
};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{JointAxis, MotorModel};
use bevy_renet::renet::RenetServer;
use sabi::prelude::*;

use sabi::stage::{NetworkCoreStage, NetworkSimulationAppExt};

use serde::{Deserialize, Serialize};

use iyes_loopless::{condition::IntoConditionalSystem, prelude::*};

use crate::attach::{Attach, AttachPlugin};
use crate::physics::{GRAB_GROUPING, REST_GROUPING};

use super::controller::{ConnectedEntities, LookTransform};
use super::input::PlayerInput;
use super::prelude::*;

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
                commands
                    .entity(joint.parent)
                    .insert(JointChildren(vec![entity]));
            }
        }
    }
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
            &mut GrabbedEntities,
        ),
        With<Hand>,
    >,
    grab_joints: Query<&GrabJoint>,
) {
    for (hand, grabbing, global, children, connected, mut grabbed) in &mut hands {
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
                    let stiffness = 0.0;
                    let damping = 0.0;
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
                            .spawn()
                            .insert(ImpulseJoint::new(other_collider, grab_joint))
                            .insert(GrabJoint);
                    });

                    grabbed.insert(other_collider);
                }
            }
        } else {
            // clean up joints if we aren't grabbing anymore
            if let Some(children) = children {
                for child in children.iter() {
                    if grab_joints.get(*child).is_ok() {
                        commands.entity(*child).despawn_recursive();
                        grabbed.remove(&*child);
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, Component, Clone, Copy)]
pub struct TargetPosition {
    pub translation: Option<Vec3>,
    pub rotation: Option<Quat>,
}

#[derive(Debug, Component, Clone, Copy)]
pub struct Grabbing(pub bool);

pub fn player_grabby_hands(
    inputs: Query<(
        &GlobalTransform,
        &LookTransform,
        &PlayerInput,
        &PlayerCamera,
    )>,
    mut impulses: Query<&mut ExternalImpulse>,
    globals: Query<&GlobalTransform>,
    joints: Query<(&GlobalTransform, &Velocity, &ImpulseJoint)>,
    mut hands: Query<(Entity, &mut Grabbing, &mut CollisionGroups, &ArmId), With<Hand>>,
    mut lines: ResMut<DebugLines>,
) {
    for (hand_entity, mut grabbing, mut collision_groups, arm_id) in &mut hands {
        let (hand_global, hand_velocity, hand_joint) = if let Ok(joint) = joints.get(hand_entity) {
            joint
        } else {
            warn!("hand does not have a joint/velocity/global");
            continue;
        };

        let arm_entity = hand_joint.parent;
        let (arm_global, arm_velocity, arm_joint) = if let Ok(joint) = joints.get(arm_entity) {
            joint
        } else {
            warn!("arm does not have a joint/velocity/global");
            continue;
        };

        let player_entity = arm_joint.parent;
        let (player_global, direction, input, camera_entity) =
            if let Ok(input) = inputs.get(player_entity) {
                input
            } else {
                warn!("player does not have an input/direction/global");
                continue;
            };

        let camera_global = if let Ok(global) = globals.get(camera_entity.0) {
            global
        } else {
            warn!("camera does not have an global");
            continue;
        };

        let arm_transform = arm_global.compute_transform();
        let shoulder = arm_transform * arm_joint.data.local_anchor2();

        let hand_transform = hand_global.compute_transform();
        let hand = hand_global.translation();
        let arm_dir = (hand - shoulder).normalize_or_zero();

        let camera = camera_global.translation();
        let camera_dir = (shoulder - camera).normalize_or_zero();

        lines.line_colored(
            shoulder,
            shoulder + camera_dir,
            crate::TICK_RATE.as_secs_f32(),
            Color::BLUE,
        );

        lines.line_colored(
            shoulder,
            shoulder + arm_dir,
            crate::TICK_RATE.as_secs_f32(),
            Color::RED,
        );

        lines.line_colored(
            shoulder,
            shoulder + camera_dir,
            crate::TICK_RATE.as_secs_f32(),
            Color::BLUE,
        );

        if input.grabby_hands(arm_id.0) {
            grabbing.0 = true;

            const STRENGTH: f32 = 0.4;
            const MAX_IMPULSE: f32 = 0.1;
            const MAX_TORQUE: f32 = 10.0;

            if let Ok(mut hand_impulse) = impulses.get_mut(hand_entity) {
                let current_dir = hand_transform.rotation * -Vec3::Y;
                let desired_dir = camera_dir;
                let desired_axis = current_dir.normalize().cross(desired_dir.normalize());

                let local_velocity = hand_velocity.linvel - arm_velocity.linvel;
                let local_angular_velocity = hand_velocity.angvel - arm_velocity.angvel;

                let hand_strength = STRENGTH;
                let hand_damping = hand_strength / 10.0;
                let wrist_strength = STRENGTH * 1.2;
                let wrist_damping = 0.04;

                let hand_spring = ((camera_dir - arm_dir).normalize_or_zero() * hand_strength)
                    - (local_velocity * hand_damping);
                let wrist_spring = desired_axis.normalize_or_zero() * wrist_strength;
                hand_impulse.impulse = hand_spring.clamp_length_max(MAX_IMPULSE);
                hand_impulse.torque_impulse = wrist_spring.clamp_length_max(MAX_TORQUE);
            }

            if let Ok(mut arm_impulse) = impulses.get_mut(arm_entity) {
                let current_dir = arm_transform.rotation * -Vec3::Y;
                let desired_dir = camera_dir;
                let desired_axis = current_dir.normalize().cross(desired_dir.normalize());

                let arm_strength = STRENGTH;
                let back_strength = STRENGTH;
                let arm_spring = ((camera_dir - arm_dir).normalize_or_zero() * arm_strength);
                let back_spring = desired_axis.normalize_or_zero() * back_strength;
                arm_impulse.impulse = arm_spring.clamp_length_max(MAX_IMPULSE);
                arm_impulse.torque_impulse = back_spring.clamp_length_max(MAX_TORQUE);
            }

            *collision_groups = GRAB_GROUPING;
        } else {
            grabbing.0 = false;
            *collision_groups = REST_GROUPING;
        }
    }
}