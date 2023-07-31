use std::fmt::Debug;

use bevy::input::mouse::MouseMotion;

use bevy::prelude::*;

use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{Isometry, JointAxesMask, JointAxis, MotorModel};

use crate::prelude::*;

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

        app.add_systems(Update, auto_aim_debug_lines);

        app.add_systems(
            FixedUpdate,
            (
                tense_arms,
                arm_target_position,
                auto_aim_pull,
                //twist_grab,
                //update_grab_sphere,
                grab_collider,
                //update_hand_collision_groups,
                grab_joint,
                last_active_arm,
            )
                .chain()
                .in_set(GrabSet)
                .in_set(FixedSet::Update)
                .before(PhysicsSet::SyncBackend),
        );
    }
}

#[derive(Default, Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
#[reflect(Component)]
pub struct GrabJoint;

pub fn grab_joint(
    mut commands: Commands,
    grabbers: Query<(Entity, &Grabbing, Option<&Children>, &GlobalTransform)>,
    mut transforms: Query<&mut Transform>,
    grab_joints: Query<(Entity, &ImpulseJoint), With<GrabJoint>>,

    globals: Query<&GlobalTransform>,
) {
    for (grabber, grabbing, children, global) in &grabbers {
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
                local_grab_point,
                global_grab_point,
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

                let local_grabber_point = if let Ok([grabbed_global, grabber_global]) =
                    globals.get_many([grabbed_entity, grabber])
                {
                    //let global_grab_point = grabbed_global.affine().transform_point3(grab_point);
                    info!("worldspace grab: {:.1?}", global_grab_point);
                    let local_grab_point = grabber_global
                        .affine()
                        .inverse()
                        .transform_point3(global_grab_point);
                    info!("localspace grab: {:.1?}", local_grab_point);
                    local_grab_point
                } else {
                    Vec3::ZERO
                };

                let motor_model = MotorModel::ForceBased;
                let max_force = 5000.0;
                let stiffness = 100.0;
                let damping = 0.4;
                let mut grab_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_SPHERICAL_AXES)
                    .local_anchor1(local_grab_point)
                    // use the center of the hand instead of exact grab point
                    .local_anchor2(Vec3::ZERO)
                    //.local_anchor2(local_grabber_point)
                    .motor_model(JointAxis::X, motor_model)
                    .motor_model(JointAxis::Y, motor_model)
                    .motor_model(JointAxis::Z, motor_model)
                    .motor_model(JointAxis::AngX, motor_model)
                    .motor_model(JointAxis::AngY, motor_model)
                    /*
                    .motor_max_force(JointAxis::X, max_force)
                    .motor_max_force(JointAxis::Y, max_force)
                    .motor_max_force(JointAxis::Z, max_force)
                    .motor_max_force(JointAxis::AngX, max_force)
                    .motor_max_force(JointAxis::AngY, max_force)
                    .motor_max_force(JointAxis::AngZ, max_force)
                    .motor_position(JointAxis::X, 0.0, stiffness, damping)
                    .motor_position(JointAxis::Z, 0.0, stiffness, damping)
                    .motor_position(JointAxis::Y, 0.0, stiffness, damping)
                    */
                    .motor_position(JointAxis::AngX, 0.0, stiffness, damping)
                    .motor_position(JointAxis::AngZ, 0.0, stiffness, damping)
                    .motor_position(JointAxis::AngY, 0.0, stiffness, damping)
                    .build();
                grab_joint.set_contacts_enabled(false);

                let mut start_joint = grab_joint.clone();
                start_joint.set_local_anchor2(local_grabber_point);

                commands.entity(grabber).with_children(|children| {
                    children
                        .spawn(ImpulseJoint::new(grabbed_entity, grab_joint))
                        .insert(JointInterpolation {
                            start: start_joint,
                            end: grab_joint,
                            over: 0.2,
                            ..default()
                        })
                        .insert(GrabJoint)
                        .insert(Name::new("Grab Joint"));
                });
            }
            None => {
                if let Some((joint_entity, _)) = joint {
                    commands
                        .entity(joint_entity)
                        .remove::<ImpulseJoint>()
                        .remove::<GrabJoint>();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct GrabSensor(pub Entity);

pub fn grab_collider(
    ctx: Res<RapierContext>,
    names: Query<DebugName>,
    globals: Query<&GlobalTransform>,
    auto_aim: Query<&AutoAim>,
    rigid_bodies: Query<Entity, With<RigidBody>>,
    parents: Query<&Parent>,
    joints: Query<&ImpulseJoint>,

    colliders: Query<(&GlobalTransform, &Collider)>,
    mut grabbers: Query<(
        Entity,
        &mut Grabbing,
        &GrabSensor,
        &GlobalTransform,
        &CharacterEntities,
    )>,
) {
    for (entity, mut grabbing, sensor, global, character) in &mut grabbers {
        if grabbing.trying_grab {
            // Don't replace the grabbed entity if we already have one grabbed.
            if grabbing.grabbed.is_some() {
                continue;
            }

            for (e1, e2, intersecting) in ctx.intersections_with(sensor.0) {
                if !intersecting {
                    continue;
                }

                let other_entity = if e1 == sensor.0 {
                    e2
                } else if e2 == sensor.0 {
                    e1
                } else {
                    continue;
                };

                let Ok([(sensor_global, sensor_collider), (other_global, other_collider)]) = colliders.get_many([sensor.0, other_entity]) else { continue };

                let sensor_transform = sensor_global.compute_transform();
                let sensor_iso = Isometry {
                    translation: sensor_transform.translation.into(),
                    rotation: sensor_transform.rotation.into(),
                };
                let other_transform = other_global.compute_transform();
                let other_iso = Isometry {
                    translation: other_transform.translation.into(),
                    rotation: other_transform.rotation.into(),
                };

                if let Ok(Some(collider_contact)) = bevy_rapier3d::parry::query::contact(
                    &sensor_iso,
                    sensor_collider.raw.as_ref(),
                    &other_iso,
                    other_collider.raw.as_ref(),
                    0.0,
                ) {
                    let global_anchor: Vec3 = collider_contact.point2.into();

                    info!("contact: {:?}", collider_contact.point2);

                    let root_entity = ctx.collider_parent(other_entity).unwrap_or(other_entity);

                    // convert back to local space.
                    let other_transform = other_global.compute_transform();
                    let local_anchor = other_global
                        .affine()
                        .inverse()
                        .transform_point3(global_anchor)
                        * other_transform.scale;

                    info!("grabbing {:?}", names.get(root_entity));

                    grabbing.grabbed = Some(Grabbed {
                        entity: root_entity,
                        local_grab_point: local_anchor.into(),
                        global_grab_point: global_anchor.into(),
                        teleport_entity: false,
                    });
                    break;
                }
            }

            /*
                       for contact_pair in rapier_context.intersections_with(grab_sensor) {
                           let other_collider = if contact_pair.collider1() == hand {
                               contact_pair.collider2()
                           } else {
                               contact_pair.collider1()
                           };

                           let other_rigidbody = if let Some(entity) = ctx.collider_parent(other_collider) {
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
                           if let Ok(other_global) = globals.get(other_collider) {
                               let auto_anchor = if let Ok(auto_aim) = auto_aim.get(other_rigidbody) {
                                   let closest_point = auto_aim.closest_point(other_global, closest_point);
                                   closest_point
                               } else {
                                   None
                               };

                               let global_anchor = if let Some(auto_anchor) = auto_anchor {
                                   auto_anchor
                               } else {
                                   closest_point
                               };

                               // convert back to local space.
                               let other_transform = other_global.compute_transform();
                               let local_anchor = other_global
                                   .affine()
                                   .inverse()
                                   .transform_point3(global_anchor)
                                   * other_transform.scale;

                               info!("grabbing {:?}", names.get(other_rigidbody));

                               grabbing.grabbed = Some(Grabbed {
                                   entity: other_rigidbody,
                                   local_grab_point: local_anchor,
                                   global_grab_point: global_anchor,
                                   teleport_entity: false,
                               });
                           }
                       }

            */
        } else {
            if let Some(grabbed) = grabbing.grabbed.take() {
                info!("dropping {:?}", names.get(grabbed.entity));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
pub struct Grabbed {
    /// Entity currently grabbed onto.
    pub entity: Entity,
    /// Local-space point where the grabbed entity is being grabbed.
    pub local_grab_point: Vec3,
    /// Global-space point where the grabbed entity is being grabbed.
    pub global_grab_point: Vec3,
    /// Entity should be teleported to a position near the grab point.
    ///
    /// Typically this is from the inventory so we don't jerk the player around.
    pub teleport_entity: bool,
}

#[derive(Default, Debug, Component, Clone, Copy, Reflect)]
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

#[derive(Default, Debug, Clone, Copy, Reflect)]
pub struct Sphere {
    /// Center of sphere in world space.
    pub center: Vec3,
    /// Radius of the sphere.
    pub radius: f32,
}

#[derive(Debug, Component, Clone, Copy, Reflect)]
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
            if !kb.pressed(KeyCode::ControlRight) {
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
            /*
            lines.line_colored(
                global_anchor,
                global_anchor + Vec3::Y,
                crate::TICK_RATE.as_secs_f32(),
                Color::YELLOW,
            );
            */

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

        /*
               lines.line_colored(
                   midpoint,
                   midpoint + Vec3::Y,
                   crate::TICK_RATE.as_secs_f32(),
                   Color::RED,
               );

               lines.line_colored(min, max, crate::TICK_RATE.as_secs_f32(), Color::PURPLE);
        */
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

pub fn arm_target_position(
    globals: Query<&GlobalTransform>,
    grab_sphere: Query<&GrabSphere>,
    mut transforms: Query<(&mut Transform, &PullOffset)>,
    inputs: Query<(&PlayerInput, &PlayerCamera, &PlayerNeck)>,
    upper_arm: Query<Entity, With<UpperArm>>,
    parents: Query<&Parent>,
    joints: Query<&ImpulseJoint>,
    mut hands: Query<(Entity, &mut Grabbing, &ArmId, &MuscleIKTarget), With<Hand>>,
) {
    for (hand_entity, mut grabbing, arm_id, muscle_ik_target) in &mut hands {
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
        } else {
            grabbing.trying_grab = false;
        }
    }
}

pub fn update_hand_collision_groups(
    mut hands: Query<(&mut CollisionGroups, &Grabbing), With<Hand>>,
) {
    for (mut groups, grabbing) in &mut hands {
        if grabbing.trying_grab && grabbing.grabbed.is_none() {
            if *groups != GRAB_GROUPING {
                *groups = GRAB_GROUPING;
            }
        } else {
            if *groups != REST_GROUPING {
                *groups = REST_GROUPING;
            }
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

#[derive(Component, Reflect, Default)]
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

#[derive(Reflect)]
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

pub fn auto_aim_debug_lines(auto_aim: Query<(&GlobalTransform, &AutoAim)>, mut gizmos: Gizmos) {
    for (global, auto) in &auto_aim {
        for primitive in &auto.0 {
            match *primitive {
                AimPrimitive::Point(point) => {
                    let center = global.transform_point(point);

                    gizmos.sphere(center, Quat::IDENTITY, 0.1, Color::LIME_GREEN);
                }
                AimPrimitive::Line { start, end } => {
                    let start = global.transform_point(start);
                    let end = global.transform_point(end);

                    gizmos.line(start, end, Color::LIME_GREEN);
                }
            }
        }
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct PullOffset(Vec3);

pub fn auto_aim_pull(
    pullers: Query<(&GlobalTransform, &AutoAim)>,
    mut offsets: Query<(&GlobalTransform, &mut PullOffset)>,
) {
    for (offset_global, mut offset) in &mut offsets {
        let mut pulls: Vec<Vec3> = Vec::new();
        for (puller_global, auto_aim) in &pullers {
            let offset_point = offset_global.translation();
            let closest = auto_aim.closest_points(puller_global, offset_point);

            for closest_point in closest {
                let difference = closest_point - offset_point;

                if difference.length() < 5.0 {
                    /*
                    lines.line_colored(
                        offset_point,
                        closest_point,
                        crate::TICK_RATE.as_secs_f32(),
                        Color::GREEN,
                    ); */
                    pulls.push(difference);
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
