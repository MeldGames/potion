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

use crate::{
    physics::{Muscle, GRAB_GROUPING, REST_GROUPING},
    FixedSet,
    prelude::*,
};

use super::input::PlayerInput;
use super::prelude::*;

pub struct GrabPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrabSet;

impl Plugin for GrabPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AutoAim>()
            .register_type::<GrabJoint>()
            .register_type::<Option<Grabbed>>()
            .register_type::<Grabbed>()
            .register_type::<Grabbing>();

        app.add_systems(
            (
                auto_aim_debug_lines,
                auto_aim_pull,
                twist_grab,
                update_grab_sphere,
                grab_collider,
                grab_joint,
                last_active_arm,
            )
                .in_set(GrabSet)
                .in_set(FixedSet::Update)
                .in_schedule(CoreSchedule::FixedUpdate),
        );

        app.add_system(
            player_extend_arm
                .in_set(GrabSet)
                .after(super::controller::player_movement)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
        app.add_system(
            tense_arms
                .in_set(GrabSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

#[derive(
    Default,
    Component,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Reflect,
    FromReflect,
)]
#[reflect(Component)]
pub struct GrabJoint;

pub fn grab_joint(
    mut commands: Commands,
    grabbers: Query<(
        Entity,
        &Grabbing,
        Option<&Children>,
        &GlobalTransform,
        &Collider,
    )>,
    mut transforms: Query<&mut Transform>,
    grab_joints: Query<(Entity, &ImpulseJoint), With<GrabJoint>>,
) {
    for (grabber, grabbing, children, global, collider) in &grabbers {
        let joint = if let Some(children) = children {
            let mut joint = None;
            for child in children.iter() {
                if let Ok(grab_joint) = grab_joints.get(*child) {
                    joint = Some(grab_joint);
                    break;
                }
            }
            joint
        } else {
            None
        };

        match grabbing.grabbed {
            Some(Grabbed {
                entity: grabbed_entity,
                grab_point,
                teleport_entity,
            }) => {
                if let Some((joint_entity, joint)) = joint {
                    if grabbed_entity == joint.parent {
                        //info!("grab joint already exists");
                        continue;
                    } else {
                        //info!("joint entity {:?} isnt the same as the grabbed {:?}, cleaning up", joint_entity, grabbed_entity);
                        commands
                            .entity(joint_entity)
                            .remove::<ImpulseJoint>()
                            .remove::<GrabJoint>();
                    }
                }

                if teleport_entity {
                    if let Ok(mut transform) = transforms.get_mut(grabbed_entity) {
                        transform.translation = global.translation();
                    }
                }

                info!("adding grab joint");
                let motor_model = MotorModel::ForceBased;
                let max_force = 1000.0;
                let stiffness = 20.0;
                let damping = 0.4;
                let mut grab_joint = SphericalJointBuilder::new()
                    .local_anchor1(grab_point)
                    .local_anchor2(Vec3::ZERO)
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

                commands.entity(grabber).with_children(|children| {
                    children
                        .spawn(ImpulseJoint::new(grabbed_entity, grab_joint))
                        .insert(GrabJoint)
                        .insert(Name::new("Grab Joint"));
                });
            }
            None => {
                if let Some((joint_entity, _)) = joint {
                    info!("cleaning up joint entity");
                    commands
                        .entity(joint_entity)
                        .remove::<ImpulseJoint>()
                        .remove::<GrabJoint>();
                }
            }
        }
    }
}

pub fn grab_collider(
    mut commands: Commands,
    name: Query<DebugName>,
    rapier_context: Res<RapierContext>,
    globals: Query<&GlobalTransform>,
    auto_aim: Query<&AutoAim>,
    rigid_bodies: Query<Entity, With<RigidBody>>,
    parents: Query<&Parent>,
    joints: Query<&ImpulseJoint>,
    mut hands: Query<
        (
            Entity,
            &mut Grabbing,
            &GlobalTransform,
            Option<&Children>,
            &CharacterEntities,
        ),
        With<Hand>,
    >,
) {
    for (hand, mut grabbing, global, children, character) in &mut hands {
        if grabbing.trying_grab {
            // Don't replace the grabbed entity if we already have one grabbed.
            if grabbing.grabbed.is_some() {
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

                if character.contains(&other_rigidbody) {
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
                    let anchor2 = Vec3::ZERO; // use the center of the hand instead of exact grab point

                    let auto_anchor = if let Ok(auto_aim) = auto_aim.get(other_rigidbody) {
                        let closest_point = auto_aim.closest_point(other_global, closest_point);
                        closest_point
                    } else {
                        None
                    };

                    let anchor1 = if let Some(auto_anchor) = auto_anchor {
                        auto_anchor
                    } else {
                        closest_point
                    };

                    // convert back to local space.
                    let other_transform = other_global.compute_transform();
                    let other_matrix = other_global.compute_matrix();
                    let anchor1 =
                        other_matrix.inverse().transform_point3(anchor1) * other_transform.scale;

                    let name = name.get(other_rigidbody).unwrap();
                    info!("grabbing {:?}", name);

                    grabbing.grabbed = Some(Grabbed {
                        entity: other_rigidbody,
                        grab_point: anchor1,
                        teleport_entity: false,
                    });
                }
            }
        } else {
            if let Some(grabbed) = grabbing.grabbed.take() {
                info!("letting go of {:?}", name.get(grabbed.entity).unwrap());
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Reflect, FromReflect)]
pub struct Grabbed {
    /// Entity currently grabbed onto.
    pub entity: Entity,
    /// Local space point where the grabbed entity is being grabbed.
    pub grab_point: Vec3,
    /// Entity should be teleported to a position near the grab point.
    ///
    /// Typically this is from the inventory so we don't jerk the player around.
    pub teleport_entity: bool,
}

#[derive(Default, Debug, Component, Clone, Copy, Reflect, FromReflect)]
pub struct Grabbing {
    /// Attempting to grab something.
    pub trying_grab: bool,
    /// Which entity we have a hold of currently.
    pub grabbed: Option<Grabbed>,

    /// Rotation on the grab sphere we are currently oriented on.
    /// Currently unused
    pub rotation: Quat,
    pub point: Vec3,
}

#[derive(Default, Debug, Clone, Copy, Reflect, FromReflect)]
pub struct Sphere {
    /// Center of sphere in world space.
    pub center: Vec3,
    /// Radius of the sphere.
    pub radius: f32,
}

#[derive(Debug, Component, Clone, Copy, Reflect, FromReflect)]
#[reflect(Component)]
pub struct GrabSphere {
    pub sphere: Option<Sphere>,
}

impl Default for GrabSphere {
    fn default() -> Self {
        Self { sphere: None }
    }
}

pub fn tense_arms(
    hands: Query<(&Grabbing, &Forearm), With<Hand>>,
    mut muscles: Query<&mut Muscle>,
    joints: Query<&ImpulseJoint>,
) {
    for (grabbing, forearm) in &hands {
        let mut entity = forearm.0;
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
    //mut impulses: Query<&mut ExternalImpulse>,
) {
    let cumulative_delta: Vec2 = mouse_motion.iter().map(|event| event.delta).sum();

    for mut grabbing in &mut grabbing {
        /*
        if let Some(grabbed_entity) = grabbing.grabbing {
            if let Ok(mut grabbed) = impulses.get_mut(grabbed_entity) {
                let axis1 = Vec3::X;
                let axis2 = Vec3::Y;
                let world1 = Quat::from_axis_angle(axis1, cumulative_delta.y / 90.0);
                let world2 = Quat::from_axis_angle(axis2, cumulative_delta.x / 90.0);

                let axis1_impulse = world1.to_scaled_axis();
                let axis2_impulse = world2.to_scaled_axis();

                //grabbed.torque_impulse += axis1_impulse;
                //grabbed.torque_impulse += axis2_impulse;
            }
        }
        */

        if grabbing.grabbed.is_none() {
            grabbing.rotation = Quat::IDENTITY;
        } else {
            if !kb.pressed(KeyCode::RControl) {
                continue;
            }

            let axis1 = Vec3::X;
            let axis2 = Vec3::Y;
            let world1 = Quat::from_axis_angle(axis1, cumulative_delta.y / 90.0);
            let world2 = Quat::from_axis_angle(axis2, cumulative_delta.x / 90.0);
            grabbing.rotation = world1 * grabbing.rotation;
            grabbing.rotation = world2 * grabbing.rotation;
        }
    }
}

pub fn update_grab_sphere(
    mut grab_spheres: Query<(Entity, &mut GrabSphere)>,
    mut grabbing: Query<&mut Grabbing>,
    parents: Query<&Parent>,
    children: Query<&Children>,
    joint_children: Query<&JointChildren>,
    grab_joints: Query<(Entity, &ImpulseJoint), With<GrabJoint>>,
    globals: Query<&GlobalTransform>,

    mut lines: ResMut<DebugLines>,
) {
    for (entity, mut grab_sphere) in &mut grab_spheres {
        let mut anchors = Vec::new();

        let results = find_children_with(&grab_joints, &children, &joint_children, entity);
        for (joint_entity, joint) in &results {
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
                global_anchor,
                global_anchor + Vec3::Y,
                crate::TICK_RATE.as_secs_f32(),
                Color::YELLOW,
            );

            anchors.push((grabber, global_anchor));
        }

        if anchors.len() == 0 {
            grab_sphere.sphere = None;
            continue;
        }

        let (min, max) = if let Some(initial) = anchors.get(0) {
            let min: Vec3 = anchors.iter().fold(initial.1, |a, b| a.min(b.1));
            let max: Vec3 = anchors.iter().fold(initial.1, |a, b| a.max(b.1));
            (min, max)
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        };

        let diameter = min.distance(max);
        let radius = diameter / 2.0;

        let midpoint = min * 0.5 + max * 0.5;
        grab_sphere.sphere = Some(Sphere {
            center: midpoint,
            radius: radius,
        });

        lines.line_colored(
            midpoint,
            midpoint + Vec3::Y,
            crate::TICK_RATE.as_secs_f32(),
            Color::RED,
        );

        lines.line_colored(min, max, crate::TICK_RATE.as_secs_f32(), Color::PURPLE);

        for (grabber, anchor) in &anchors {
            let mut grabbing = if let Ok(grabbing) = grabbing.get_mut(*grabber) {
                grabbing
            } else {
                continue;
            };

            grabbing.point = *anchor;
        }
    }
}

pub fn player_extend_arm(
    globals: Query<&GlobalTransform>,
    grab_sphere: Query<&GrabSphere>,
    mut transforms: Query<(&mut Transform, &PullOffset)>,
    inputs: Query<(&PlayerInput, &PlayerCamera, &PlayerNeck)>,
    upper_arm: Query<Entity, With<UpperArm>>,
    parents: Query<&Parent>,
    joints: Query<&ImpulseJoint>,
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
            warn!("couldn't find camera global");
            continue;
        };

        let neck_global = if let Ok(global) = globals.get(neck.0) {
            global
        } else {
            warn!("couldn't find neck global");
            continue;
        };

        let direction =
            (neck_global.translation() - camera_global.translation()).normalize_or_zero();

        if input.extend_arm(arm_id.0) {
            if let Ok((mut target_position, pull_offset)) = transforms.get_mut(muscle_ik_target.0) {
                let _neck_yaw = Quat::from_axis_angle(Vec3::Y, input.yaw as f32);

                let upper_arm =
                    find_parent_with(&upper_arm, &parents, &joints, hand_entity).unwrap();
                let Ok(joint) = joints.get(upper_arm) else { continue };
                let Ok(upper_global) = globals.get(upper_arm) else { continue };
                let shoulder = joint.data.local_anchor2();
                let shoulder_worldspace = upper_global.transform_point(shoulder);

                let grab_sphere =
                    find_parent_with(&grab_sphere, &parents, &joints, hand_entity).unwrap();
                //info!("grab sphere: {:?}", grab_sphere);

                let _sphere_offset = if let Some(_sphere) = grab_sphere.sphere {
                    //let relative = sphere.center - grabbing.point;
                    //target_position.translation = sphere.center;// + grabbing.rotation// * relative;
                } else {
                };

                target_position.translation = shoulder_worldspace + direction * 2.0;

                if grabbing.grabbed.is_none() {
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

#[derive(Component, Debug, Clone)]
pub struct PrevGrabbing(pub Grabbing);

pub fn last_active_arm(
    mut commands: Commands,
    mut prev_grabbing: Query<&mut PrevGrabbing>,
    mut hands: Query<(Entity, &Grabbing, &mut LastActive), With<Hand>>,
) {
    for (hand_entity, grabbing, mut last_active) in &mut hands {
        if let Ok(mut prev_grabbing) = prev_grabbing.get_mut(hand_entity) {
            if !prev_grabbing.0.trying_grab && grabbing.trying_grab {
                last_active.0 = std::time::Instant::now();
            }

            prev_grabbing.0 = grabbing.clone();
        } else {
            commands
                .entity(hand_entity)
                .insert(PrevGrabbing(grabbing.clone()));
        }
    }
}

#[derive(Component, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct AutoAim(pub Vec<AimPrimitive>);

impl AutoAim {
    pub fn closest_points(&self, global: &GlobalTransform, point: Vec3) -> Vec<Vec3> {
        self.0
            .iter()
            .map(|primitive| primitive.closest_point(global, point))
            .collect()
    }

    pub fn closest_point(&self, global: &GlobalTransform, point: Vec3) -> Option<Vec3> {
        let mut points = self.closest_points(global, point);
        points.sort_by(|a, b| a.length().total_cmp(&b.length()));
        points.get(0).cloned()
    }
}

#[derive(Reflect, FromReflect)]
pub enum AimPrimitive {
    Point(Vec3),
    Line { start: Vec3, end: Vec3 },
}

impl Default for AimPrimitive {
    fn default() -> Self {
        Self::Point(Vec3::ZERO)
    }
}

impl AimPrimitive {
    pub fn closest_point(&self, global: &GlobalTransform, point: Vec3) -> Vec3 {
        match *self {
            Self::Point(auto_point) => global.transform_point(auto_point),
            Self::Line { start, end } => {
                let start = global.transform_point(start);
                let end = global.transform_point(end);

                let vector = end - start;
                let direction = vector.normalize_or_zero();
                let relative_point = point - start;
                let distance = direction.dot(relative_point);

                if distance < 0.0 {
                    start
                } else if distance > vector.length() {
                    end
                } else {
                    start + direction * distance
                }
            }
        }
    }
}

pub fn auto_aim_debug_lines(
    auto_aim: Query<(&GlobalTransform, &AutoAim)>,
    mut lines: ResMut<DebugLines>,
) {
    for (global, auto) in &auto_aim {
        for primitive in &auto.0 {
            match *primitive {
                AimPrimitive::Point(point) => {
                    let point = global.transform_point(point);

                    lines.line_colored(
                        point,
                        point + Vec3::Y * 0.25,
                        crate::TICK_RATE.as_secs_f32(),
                        Color::LIME_GREEN,
                    );
                }
                AimPrimitive::Line { start, end } => {
                    let start = global.transform_point(start);
                    let end = global.transform_point(end);

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
}

#[derive(Default, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct PullOffset(Vec3);

pub fn auto_aim_pull(
    pullers: Query<(&GlobalTransform, &AutoAim)>,
    mut offsets: Query<(&GlobalTransform, &mut PullOffset)>,

    mut lines: ResMut<DebugLines>,
) {
    for (offset_global, mut offset) in &mut offsets {
        let mut pulls: Vec<Vec3> = Vec::new();
        for (puller_global, auto_aim) in &pullers {
            let offset_point = offset_global.translation();
            let closest = auto_aim.closest_points(puller_global, offset_point);

            for closest_point in closest {
                let difference = closest_point - offset_point;

                if difference.length() < 5.0 {
                    lines.line_colored(
                        offset_point,
                        closest_point,
                        crate::TICK_RATE.as_secs_f32(),
                        Color::GREEN,
                    );
                    //pulls.push(difference);
                }
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

#[cfg(test)]
mod auto_aim_test {
    use bevy::prelude::*;

    #[test]
    fn closest_point_on_line() {
        let start = Vec3::new(3.0, 3.0, 3.0);
        let end = Vec3::new(4.0, 4.0, 4.0);

        let _point = Vec3::new(3.0, 3.0, 3.0);

        let closest_point = |point: Vec3| -> Vec3 {
            let vector = end - start;
            let direction = vector.normalize_or_zero();
            let relative_point = point - start;
            let distance = direction.dot(relative_point);

            println!("----");
            println!("relative {:?}", relative_point);
            println!("dir {:?}", direction);
            println!("length: {:?}", vector.length());
            println!("distance: {:?}", distance);

            if distance < 0.0 {
                start
            } else if distance > vector.length() {
                end
            } else {
                start + direction * distance
            }
        };

        println!("{:?}", closest_point(Vec3::new(3.0, 3.0, 3.0)));
        println!("{:?}", closest_point(Vec3::new(3.5, 3.0, 3.0)));
        //println!("{:?}", closest_point(Vec3::new(3.1, 3.1, 3.1)));
        //println!("{:?}", closest_point(Vec3::new(4.1, 4.1, 4.1)));
    }
}
