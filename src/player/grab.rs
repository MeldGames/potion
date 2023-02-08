use std::fmt::Debug;

use bevy::{
    ecs::{
        entity::Entities,
        query::{ReadOnlyWorldQuery, WorldQuery},
    },
    input::mouse::MouseMotion,
};

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_debug_lines::DebugLines;

use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{JointAxis, MotorModel};

use crate::physics::{Muscle, GRAB_GROUPING, REST_GROUPING};

use super::controller::ConnectedEntities;
use super::input::PlayerInput;
use super::prelude::*;

pub struct GrabPlugin;

impl Plugin for GrabPlugin {
    fn build(&self, app: &mut App) {
        use sabi::stage::NetworkSimulationAppExt;

        app.register_type::<AutoAim>();

        app.add_network_system(auto_aim_debug_lines);
        app.add_network_system(auto_aim_pull);
        app.add_network_system(twist_grab);
    }
}

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

pub fn grab_collider(
    mut commands: Commands,
    name: Query<&Name>,
    rapier_context: Res<RapierContext>,
    globals: Query<&GlobalTransform>,
    rigid_bodies: Query<Entity, With<RigidBody>>,
    parents: Query<&Parent>,
    joints: Query<&ImpulseJoint>,
    mut hands: Query<
        (
            Entity,
            &mut Grabbing,
            &GlobalTransform,
            Option<&Children>,
            &ConnectedEntities,
            &mut GrabbedEntities,
        ),
        With<Hand>,
    >,
    grab_joints: Query<(&ImpulseJoint, &GrabJoint)>,
) {
    for (hand, mut grabbing, global, children, connected, mut grabbed) in &mut hands {
        if grabbing.trying_grab {
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

            grabbing.grabbing = already_grabbing;

            if already_grabbing {
                continue;
            }

            for contact_pair in rapier_context.contacts_with(hand) {
                let other_collider = if contact_pair.collider1() == hand {
                    contact_pair.collider2()
                } else {
                    contact_pair.collider1()
                };

                let other_rigidbody = if let Some(entity) = find_parent_with(&rigid_bodies, &parents, &joints, other_collider) {
                    entity
                } else {
                    continue;
                };

                if connected.contains(&other_rigidbody) {
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

                if let Ok(other_global) = globals.get(other_rigidbody) {
                    // convert back to local space.
                    let other_transform = other_global.compute_transform();
                    let other_matrix = other_global.compute_matrix();
                    let anchor1 = other_matrix.inverse().project_point3(closest_point)
                        * other_transform.scale;
                    let transform = global.compute_transform();
                    let matrix = global.compute_matrix();
                    let anchor2 = matrix.inverse().project_point3(closest_point) * transform.scale;

                    if let Ok(name) = name.get(other_rigidbody) {
                        info!("grabbing {:?}", name.as_str());
                    } else {
                        info!("grabbing entity {:?}", other_rigidbody);
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
                            .spawn(ImpulseJoint::new(other_rigidbody, grab_joint))
                            .insert(GrabJoint);
                    });

                    grabbed.insert(other_rigidbody);
                }
            }
        } else {
            grabbing.grabbing = false;

            // clean up joints if we aren't grabbing anymore
            if let Some(children) = children {
                for child in children.iter() {
                    if let Ok((_impulse_joint, _joint)) = grab_joints.get(*child) {
                        commands.entity(*child).despawn_recursive();
                        grabbed.remove(&*child);
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, Component, Clone, Copy)]
pub struct Grabbing {
    pub trying_grab: bool,

    pub grabbing: bool,

    // Offset from the cameras target position
    pub target_offset: Vec3,

    pub pitch: f32,
    pub yaw: f32,
}

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
    mut muscles: Query<&mut Muscle>,
    joints: Query<&ImpulseJoint>,
) {
    for (hand_entity, grabbing) in &hands {
        let mut entity = hand_entity;
        while let Ok(mut muscle) = muscles.get_mut(entity) {
            if muscle.tense != grabbing.trying_grab {
                muscle.tense = grabbing.trying_grab;
            }

            if let Ok(joint) = joints.get(entity) {
                entity = joint.parent;
            } else {
                break;
            }
        }
    }
}

pub fn twist_grab(
    kb: Res<Input<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut grabbing: Query<&mut Grabbing>,
) {
    if !kb.pressed(KeyCode::RControl) {
        return;
    }

    let mut cumulative_delta = Vec2::ZERO;
    for motion in mouse_motion.iter() {
        cumulative_delta += motion.delta;
    }

    for mut grabbing in &mut grabbing {
        grabbing.pitch -= cumulative_delta.y / 20.0;
        grabbing.yaw -= cumulative_delta.x / 20.0;
    }
}

pub fn player_grabby_hands(
    globals: Query<&GlobalTransform>,
    mut transforms: Query<(&mut Transform, &PullOffset)>,
    inputs: Query<(&PlayerInput, &PlayerCamera, &PlayerNeck)>,
    //ik_base: Query<&IKBase>,
    parents: Query<&Parent>,
    joints: Query<&ImpulseJoint>,
    //ctx: Res<RapierContext>,
    mut hands: Query<
        (
            Entity,
            &mut Grabbing,
            &mut CollisionGroups,
            &ArmId,
            &MuscleIKTarget,
        ),
        With<Hand>,
    >,
) {
    for (hand_entity, mut grabbing, mut collision_groups, arm_id, muscle_ik_target) in &mut hands {
        let input = find_parent_with(&inputs, &parents, &joints, hand_entity);

        let (input, cam, neck) = if let Some(input) = input {
            input
        } else {
            warn!("couldn't find parent input for hand entity");
            continue;
        };

        let camera_global = if let Ok(global) = globals.get(cam.0) {
            global
        } else {
            continue;
        };

        let neck_global = if let Ok(global) = globals.get(neck.0) {
            global
        } else {
            continue;
        };

        let direction =
            (neck_global.translation() - camera_global.translation()).normalize_or_zero();

        if input.grabby_hands(arm_id.0) {
            if let Ok((mut target_position, pull_offset)) = transforms.get_mut(muscle_ik_target.0) {
                //let (_, neck_rotation, _) = neck_global.to_scale_rotation_translation();
                let neck_rotation = Quat::from_axis_angle(Vec3::Y, input.yaw as f32);

                let grab_rotation = Quat::from_axis_angle(Vec3::Z, grabbing.yaw as f32);
                    //* Quat::from_axis_angle(Vec3::X, grabbing.pitch as f32);
                let relative_offset = neck_rotation * grab_rotation * grabbing.target_offset;
                target_position.translation =
                    neck_global.translation() + direction * 2.5 + relative_offset;

                if !grabbing.grabbing && pull_offset.0.length() > 0.0 {
                    target_position.translation += pull_offset.0;
                }
            }

            grabbing.trying_grab = true;
            *collision_groups = GRAB_GROUPING;
        } else {
            grabbing.trying_grab = false;
            *collision_groups = REST_GROUPING;
        }
    }
}

#[derive(Component, Reflect, FromReflect)]
#[reflect(Component)]
pub enum AutoAim {
    Point(Vec3),
    Line { start: Vec3, end: Vec3 },
}

impl Default for AutoAim {
    fn default() -> Self {
        Self::Point(Vec3::ZERO)
    }
}

impl AutoAim {
    pub fn closest_point(&self, global: &GlobalTransform, point: Vec3) -> Vec3 {
        match *self {
            Self::Point(auto_point) => transform(global, auto_point),
            Self::Line { start: a, end: b } => {
                if (a - b).length() < 0.001 {
                    return Vec3::ZERO;
                }

                let a = transform(global, a);
                let b = transform(global, b);

                let ap = point - a;
                let ab = b - a;

                let distance = ab.dot(ap) / ab.length();

                if distance < 0.0 {
                    a
                } else if distance > 1.0 {
                    b
                } else {
                    a + ab * distance
                }
            }
        }
    }
}

fn transform(global: &GlobalTransform, point: Vec3) -> Vec3 {
    global
        .mul_transform(Transform::from_translation(point))
        .translation()
}

pub fn auto_aim_debug_lines(
    auto_aim: Query<(&GlobalTransform, &AutoAim)>,
    mut lines: ResMut<DebugLines>,
) {
    for (global, auto) in &auto_aim {
        match *auto {
            AutoAim::Point(point) => {
                let point = transform(global, point);

                lines.line_colored(
                    point,
                    point,
                    crate::TICK_RATE.as_secs_f32(),
                    Color::LIME_GREEN,
                );
            }
            AutoAim::Line { start, end } => {
                let start = transform(global, start);
                let end = transform(global, end);

                lines.line_colored(
                    start,
                    end,
                    crate::TICK_RATE.as_secs_f32(),
                    Color::LIME_GREEN,
                );
            }
        }
    }
}

#[derive(Default, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct PullOffset(Vec3);

pub fn auto_aim_pull(
    pullers: Query<(&GlobalTransform, &AutoAim)>,
    mut offsets: Query<(&GlobalTransform, &mut PullOffset)>,
) {
    for (offset_global, mut offset) in &mut offsets {
        let mut pulls = Vec::new();
        for (puller_global, auto_aim) in &pullers {
            let offset_point = offset_global.translation() - offset.0;
            let closest = auto_aim.closest_point(puller_global, offset_point);

            let difference = closest - offset_point;
            let distance = difference.length();

            if distance < 1.5 {
                pulls.push(difference);
            }
        }

        pulls.sort_by(|a, b| a.length().total_cmp(&b.length()));
        if let Some(pull) = pulls.get(0) {
            offset.0 = *pull;
        } else {
            offset.0 = Vec3::ZERO;
        }
    }
}
