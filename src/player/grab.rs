

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
        app.add_network_system(update_grab_sphere);
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrabJoint;

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
        ),
        With<Hand>,
    >,
    grab_joints: Query<(&ImpulseJoint, &GrabJoint)>,
) {
    for (hand, mut grabbing, global, children, connected) in &mut hands {
        
        if grabbing.trying_grab {
            /*
            let mut already_grabbing = None;

            if let Some(children) = children {
                for child in children.iter() {
                    if grab_joints.contains(*child) {
                        // We are already grabbing something so just skip this hand.
                        already_grabbing = Some();
                        break;
                    }
                }
            }

            grabbing.grabbing = already_grabbing;
            */

            if grabbing.grabbing.is_some() {
                continue;
            }

            for contact_pair in rapier_context.contacts_with(hand) {
                let other_collider = if contact_pair.collider1() == hand {
                    contact_pair.collider2()
                } else {
                    contact_pair.collider1()
                };

                let other_rigidbody = if let Some(entity) =
                    find_parent_with(&rigid_bodies, &parents, &joints, other_collider)
                {
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

                    grabbing.grabbing = Some(other_rigidbody);
                }
            }
        } else {
            grabbing.grabbing = None;

            // clean up joints if we aren't grabbing anymore
            if let Some(children) = children {
                for child in children.iter() {
                    if let Ok((_impulse_joint, _joint)) = grab_joints.get(*child) {
                        commands.entity(*child).despawn_recursive();
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, Component, Clone, Copy)]
pub struct Grabbing {
    pub trying_grab: bool,
    pub grabbing: Option<Entity>,
    pub rotation: Quat,
    pub dir: Vec3,
}

#[derive(Default, Debug, Component, Clone, Copy)]
pub struct GrabSphere {
    pub center: Vec3,
    pub radius: f32,
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

    let cumulative_delta: Vec2 = mouse_motion.iter().map(|event| event.delta).sum();

    for mut grabbing in &mut grabbing {
        let axis1 = Vec3::X;
        let axis2 = Vec3::Y;
        let world1 = Quat::from_axis_angle(axis1, cumulative_delta.y / 90.0);
        let world2 = Quat::from_axis_angle(axis2, cumulative_delta.x / 90.0);
        //lines.line_colored(-axis1, axis1 * 2.0, 3.0, Color::BLUE);
        //lines.line_colored(-axis2, axis2 * 2.0, 3.0, Color::GREEN);
        grabbing.rotation = world1 * grabbing.rotation;
        grabbing.rotation = world2 * grabbing.rotation;
    }
}

pub fn children_with_recursive<'a, Q: WorldQuery, F: ReadOnlyWorldQuery>(
    query: &'a Query<Q, F>,
    children: &'a Query<&Children>,
    joint_children: &'a Query<&JointChildren>,
    base: Entity,
) -> Vec<<<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>> {
    let mut results = Vec::new();
    let mut possibilities = vec![base];

    while let Some(possible) = possibilities.pop() {
        if let Ok(result) = query.get(possible) {
            results.push(result);
        }

        if let Ok(children) = children.get(possible) {
            possibilities.extend(children);
        }

        if let Ok(children) = joint_children.get(possible) {
            possibilities.extend(&children.0);
        }
    }

    results
}

pub fn update_grab_sphere(
    mut grab_spheres: Query<(Entity, &GlobalTransform, &mut GrabSphere)>,
    mut grabbing: Query<&mut Grabbing>,
    parents: Query<&Parent>,
    children: Query<&Children>,
    joint_children: Query<&JointChildren>,
    grab_joints: Query<(Entity, &ImpulseJoint), With<GrabJoint>>,
    globals: Query<&GlobalTransform>,

    mut lines: ResMut<DebugLines>,
) {
    for (entity, sphere_base, mut sphere) in &mut grab_spheres {
        let mut anchors = Vec::new();

        let results = children_with_recursive(&grab_joints, &children, &joint_children, entity);
        for (joint_entity, joint) in &results {
            //info!("grabbing: {:?}", joint.parent);
            // we need to get translate from the joints local space to the spheres local space
            let joint: &ImpulseJoint = joint;

            //let entity1 = joint.parent;
            let grabber = parents.get(*joint_entity).unwrap().get();
            let global = if let Ok(global) = globals.get(grabber) {
                global
            } else {
                continue;
            };

            let anchor = joint.data.local_anchor2();
            let global_anchor = global.transform_point(anchor);

            lines.line_colored(
                sphere_base.translation(),
                global_anchor,
                crate::TICK_RATE.as_secs_f32(),
                Color::RED,
            );

            anchors.push((grabber, sphere_base.compute_matrix().inverse().project_point3(global_anchor), ));
        }

        let (min, max) = 
            if let Some(initial) = anchors.get(0) {
                let min: Vec3 = anchors.iter().fold(initial.1, |a, b| a.min(b.1));
                let max: Vec3 = anchors.iter().fold(initial.1, |a, b| a.max(b.1));
                (min, max)
            } else {
                (Vec3::ZERO, Vec3::ZERO)
            };

        let diameter = min.distance(max);
        let radius = diameter / 2.0;

        let midpoint = min * 0.5 + max * 0.5;
        sphere.center = midpoint;
        sphere.radius = radius;

        lines.line_colored(
            sphere_base.transform_point(min),
            sphere_base.transform_point(max),
            crate::TICK_RATE.as_secs_f32(),
            Color::RED,
        );

        for (grabber, anchor) in &anchors {
            let mut grabbing = if let Ok(grabbing) = grabbing.get_mut(*grabber) {
                grabbing
            } else  {
                continue;
            };

            grabbing.dir = (sphere.center - *anchor);
        }
    }


}

pub fn player_grabby_hands(
    kb: Res<Input<KeyCode>>,
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
            &mut ExternalImpulse,
            &ArmId,
            &MuscleIKTarget,
        ),
        With<Hand>,
    >,
) {
    for (
        hand_entity,
        mut grabbing,
        mut collision_groups,
        mut hand_impulse,
        arm_id,
        muscle_ik_target,
    ) in &mut hands
    {
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
                let neck_yaw = Quat::from_axis_angle(Vec3::Y, input.yaw as f32);

                let grab_rotation = neck_yaw * grabbing.rotation;
                target_position.translation = neck_global.translation() + direction * 2.5 + grab_rotation * grabbing.dir;

                if grabbing.grabbing.is_none() {
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
