use std::fmt::Debug;

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_mod_inverse_kinematics::IkConstraint;

use std::f32::consts::PI;

use bevy_mod_wanderlust::{
    CharacterControllerBundle, ControllerPhysicsBundle, ControllerSettings, Spring,
};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{JointAxis, MotorModel};
use bevy_renet::renet::RenetServer;
use sabi::prelude::*;

use super::prelude::*;
use crate::attach::Attach;
use crate::cauldron::NamedEntity;
use crate::physics::{ContactFilter, Muscle};
use crate::DebugVisible;

#[derive(Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub id: u64,
}
#[derive(Component, Debug)]
pub struct LocalPlayer;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpperArm;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hand;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ForeArm;

#[derive(Component, Debug)]
pub struct Neck;

#[derive(Component, Debug)]
pub struct PlayerCamera(pub Entity);

#[derive(Component, Debug)]
pub struct PlayerNeck(pub Entity);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlayerEvent {
    Spawn { id: u64 },
    SetupLocal { id: u64 },
}

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: ResMut<AssetServer>,
    mut player_reader: EventReader<PlayerEvent>,

    mut lobby: ResMut<Lobby>,
    mut server: Option<ResMut<RenetServer>>,
) {
    for (event, id) in player_reader.iter_with_id() {
        info!("player event {:?}: {:?}", id, event);
        match event {
            &PlayerEvent::SetupLocal { id } => {
                let player_entity = *lobby.players.get(&id).expect("Expected a player");
                info!("setting up local entity: {:?}", player_entity);
            }
            &PlayerEvent::Spawn { id } => {
                info!("spawning player {}", id);
                let global_transform = GlobalTransform::from(Transform::from_xyz(0.0, 10.0, 0.0));

                let player_height = 1.0;
                let player_radius = 0.5;
                // Spawn player cube
                let player_entity = commands
                    .spawn(CharacterControllerBundle {
                        settings: ControllerSettings {
                            acceleration: 5.0,
                            max_speed: 7.0,
                            max_acceleration_force: 4.0,
                            up_vector: Vec3::Y,
                            gravity: -9.8,
                            max_ground_angle: 45.0 * (PI / 180.0),
                            min_float_offset: -0.3,
                            max_float_offset: 0.05,
                            jump_time: 0.5,
                            jump_initial_force: 12.0,
                            jump_stop_force: 0.01,
                            jump_decay_function: |x| (1.0 - x).sqrt(),
                            jump_skip_ground_check_duration: 0.5,
                            coyote_time_duration: 0.16,
                            jump_buffer_duration: 0.16,
                            force_scale: Vec3::new(1.0, 0.0, 1.0),
                            float_cast_length: 1.0,
                            //float_cast_length: 1.,
                            //float_cast_collider: Collider::ball(player_radius - 0.05),
                            float_cast_collider: Collider::ball(player_radius),
                            float_distance: 1.0,
                            float_spring: Spring {
                                strength: 80.0,
                                damping: 0.7,
                            },
                            upright_spring: Spring {
                                strength: 500.0,
                                damping: 0.7,
                            },
                            opposing_movement_impulse_scale: 0.0,
                            ..default()
                        },
                        physics: ControllerPhysicsBundle {
                            collider: Collider::capsule(
                                Vec3::new(0.0, 0.0, 0.0),
                                Vec3::new(0.0, player_height, 0.0),
                                player_radius,
                            ),
                            //collider: Collider::ball(player_radius),
                            //rigidbody: RigidBody::KinematicPositionBased,
                            ..default()
                        },
                        transform: global_transform.compute_transform(),
                        global_transform: global_transform,
                        ..default()
                    })
                    .insert(SceneBundle {
                        scene: asset_server.load("models/skin.gltf#Scene0"),
                        ..default()
                    })
                    //.insert(crate::deposit::Value::new(500))
                    //.insert(ColliderMassProperties::Density(5.0))
                    .insert(PlayerInput::default())
                    .insert(Player { id: id })
                    .insert(Name::new(format!("Player {}", id.to_string())))
                    .insert(ConnectedEntities::default())
                    //.insert(ConnectedMass::default())
                    .insert(Owned)
                    .insert(ReadMassProperties::default())
                    //.insert(Loader::<Mesh>::new("scenes/gltfs/boi.glb#Mesh0/Primitive0"))
                    .insert(crate::physics::PLAYER_GROUPING)
                    .insert(LookTransform::default())
                    .id();

                let distance_from_body = player_radius + 0.3;
                attach_arm(
                    &mut commands,
                    &mut meshes,
                    player_entity,
                    global_transform.compute_transform(),
                    Vec3::new(distance_from_body, player_height, 0.0),
                    0,
                );
                attach_arm(
                    &mut commands,
                    &mut meshes,
                    player_entity,
                    global_transform.compute_transform(),
                    Vec3::new(-distance_from_body, player_height, 0.0),
                    1,
                );

                // for some body horror
                /*
                attach_arm(
                    &mut commands,
                    &mut meshes,
                    player_entity,
                    global_transform.compute_transform(),
                    Vec3::new(0.0, player_height, distance_from_body),
                    2,
                );

                attach_arm(
                    &mut commands,
                    &mut meshes,
                    player_entity,
                    global_transform.compute_transform(),
                    Vec3::new(0.0, player_height, -distance_from_body),
                    3,
                );
                */

                let camera = commands
                    .spawn(TransformBundle::from_transform(
                        Transform::from_translation(Vec3::new(0., 0., 4.))
                            .looking_at(Vec3::ZERO, Vec3::Y),
                    ))
                    .insert(Camera3dBundle {
                        projection: PerspectiveProjection {
                            far: 10000.,
                            ..default()
                        }
                        .into(),
                        camera: Camera {
                            priority: 50,
                            is_active: true,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(AvoidIntersecting {
                        dir: Vec3::Z,
                        max_toi: 4.0,
                        buffer: 0.05,
                    })
                    .insert(ZoomScroll {
                        current: 8.0,
                        scroll_sensitivity: -0.5,
                        min: 4.0,
                        max: 24.0,
                    })
                    .insert(ZoomScrollForToi)
                    .insert(Name::new("Player Camera"))
                    .id();

                let head = commands
                    .spawn((
                        TransformBundle::from_transform(Transform::from_xyz(0., 1., 0.)),
                        Name::new("Head"),
                    ))
                    .insert(Velocity::default())
                    .id();

                commands.entity(player_entity).push_children(&[head]);

                let neck = commands
                    .spawn((TransformBundle::default(), Neck, Name::new("Neck")))
                    .insert(Attach::translation(head))
                    /* .insert(AttachTranslation::Spring {
                        strength: 50.0,
                        damp_ratio: 0.9,
                    }) */
                    .insert(Velocity::default())
                    .id();

                commands.entity(neck).push_children(&[camera]);
                commands
                    .entity(player_entity)
                    .insert(PlayerCamera(camera))
                    .insert(PlayerNeck(neck));

                // We could send an InitState with all the players id and positions for the client
                // but this is easier to do.

                lobby.players.insert(id, player_entity);
                if let Some(ref mut server) = server {
                    for (existing_id, existing_entity) in lobby.players.iter() {
                        let message = bincode::serialize(&ServerMessage::PlayerConnected {
                            id: *existing_id,
                            entity: (*existing_entity).into(),
                        })
                        .unwrap();

                        server.send_message(id, ServerChannel::Message.id(), message);
                    }
                }

                if let Some(ref mut server) = server {
                    let message = bincode::serialize(&ServerMessage::PlayerConnected {
                        id: id,
                        entity: player_entity.into(),
                    })
                    .unwrap();
                    server.broadcast_message(ServerChannel::Message.id(), message);

                    let message = bincode::serialize(&ServerMessage::AssignOwnership {
                        entity: player_entity.into(),
                    })
                    .unwrap();
                    server.send_message(id, ServerChannel::Message.id(), message);

                    let message = bincode::serialize(&ServerMessage::SetPlayer { id }).unwrap();
                    server.send_message(id, ServerChannel::Message.id(), message);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct ArmId(pub usize);

#[derive(Debug, Clone, Component)]
pub struct MuscleIKTarget(pub Entity);

impl MuscleIKTarget {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

#[derive(Debug, Clone, Component)]
pub struct IKBase(pub Entity);

impl IKBase {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

pub fn attach_arm(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    to: Entity,
    to_transform: Transform,
    at: Vec3,
    index: usize,
) {
    let max_force = 100.0;
    let twist_stiffness = 20.0;
    let twist_damping = 9.0;
    let resting_stiffness = 15.0;
    let resting_damping = 8.0;
    let arm_radius = 0.25;
    let hand_radius = arm_radius + 0.05;
    let motor_model = MotorModel::ForceBased;

    //let arm_height = Vec3::new(0.0, 1.25 - arm_radius - hand_radius, 0.0);
    let forearm_height = Vec3::new(0.0, 0.75 - arm_radius, 0.0);
    let upperarm_height = Vec3::new(0.0, 0.75 - arm_radius, 0.0);
    //let arm_height = forearm_height + upperarm_height;
    //let arm_height = Vec3::new(0.0, 1.25, 0.0);

    let upperarm_target = commands
        .spawn(TransformBundle::from_transform(
            Transform::from_translation(at),
        ))
        .insert(DebugVisible)
        .insert(Name::new(format!("Upperarm Target {}", index)))
        .id();

    let target = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.05,
                ..default()
            })),
            transform: Transform::from_translation(Vec3::new(0.0, 2.0, -2.0)),
            ..default()
        })
        .insert(Name::new(format!("IK Target {}", index)))
        .insert(ArmId(index))
        .insert(IKBase::new(upperarm_target))
        .insert(DebugVisible)
        .insert(PullOffset::default())
        .id();

    let forearm_target = commands
        .spawn(TransformBundle::from_transform(
            Transform::from_translation(-forearm_height),
        ))
        .insert(Name::new(format!("Forearm Target {}", index)))
        .insert(DebugVisible)
        .id();

    let pole_target = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.02,
                ..default()
            })),
            transform: Transform::from_translation(Vec3::new(-1.0, 0.4, -0.2)),
            ..default()
        })
        .insert(Name::new(format!("Elbow pole {}", index)))
        .insert(DebugVisible)
        .id();

    let hand_target = commands
        .spawn(TransformBundle::from_transform(
            Transform::from_translation(-forearm_height - Vec3::new(0.0, arm_radius, 0.0)),
        ))
        .insert(IkConstraint {
            chain_length: 2,
            iterations: 20,
            target: target,
            pole_target: Some(pole_target),
            pole_angle: std::f32::consts::FRAC_PI_2,
        })
        .insert(Name::new(format!("Hand Target {}", index)))
        .insert(DebugVisible)
        .id();

    //commands.entity(to).add_child(target);
    commands.entity(to).add_child(upperarm_target);
    commands.entity(upperarm_target).add_child(forearm_target);
    commands.entity(forearm_target).add_child(hand_target);

    let mut upperarm_joint = SphericalJointBuilder::new()
        .local_anchor1(at) // body local
        .local_anchor2(upperarm_height)
        .motor_model(JointAxis::AngX, motor_model)
        .motor_model(JointAxis::AngY, motor_model)
        .motor_model(JointAxis::AngZ, motor_model)
        .motor_max_force(JointAxis::AngX, max_force)
        .motor_max_force(JointAxis::AngY, max_force)
        .motor_max_force(JointAxis::AngZ, max_force)
        .motor_position(JointAxis::AngX, 0.0, resting_stiffness, resting_damping)
        .motor_position(JointAxis::AngZ, 0.0, resting_stiffness, resting_damping)
        .motor_position(JointAxis::AngY, 0.0, twist_stiffness, twist_damping)
        .build();
    upperarm_joint.set_contacts_enabled(false);

    let upperarm_entity = commands
        .spawn(TransformBundle::from_transform(to_transform))
        .insert(Name::new(format!("UpperArm {}", index)))
        .insert(UpperArm)
        .insert(RigidBody::Dynamic)
        .insert(ExternalImpulse::default())
        .insert(Velocity::default())
        .insert(ReadMassProperties::default())
        .insert(crate::physics::REST_GROUPING)
        .insert(Collider::capsule(Vec3::ZERO, upperarm_height, arm_radius))
        .insert(ImpulseJoint::new(to, upperarm_joint))
        .insert(ActiveHooks::MODIFY_SOLVER_CONTACTS)
        .insert(ContactFilter::default())
        .insert(ConnectedEntities::default())
        .insert(ArmId(index))
        .insert(Muscle::new(upperarm_target))
        .id();

    let mut forearm_joint = SphericalJointBuilder::new()
        .local_anchor2(forearm_height)
        .motor_model(JointAxis::AngX, motor_model)
        .motor_model(JointAxis::AngY, motor_model)
        .motor_model(JointAxis::AngZ, motor_model)
        .motor_max_force(JointAxis::AngX, max_force)
        .motor_max_force(JointAxis::AngY, max_force)
        .motor_max_force(JointAxis::AngZ, max_force)
        .motor_position(JointAxis::AngX, 0.0, resting_stiffness, resting_damping)
        .motor_position(JointAxis::AngZ, 0.0, resting_stiffness, resting_damping)
        .motor_position(JointAxis::AngY, 0.0, twist_stiffness, twist_damping)
        .build();
    forearm_joint.set_contacts_enabled(false);

    let forearm_entity = commands
        .spawn(TransformBundle::from_transform(to_transform))
        .insert(Name::new(format!("ForeArm {}", index)))
        .insert(ForeArm)
        .insert(RigidBody::Dynamic)
        .insert(ExternalImpulse::default())
        .insert(Velocity::default())
        .insert(ReadMassProperties::default())
        .insert(crate::physics::REST_GROUPING)
        .insert(Collider::capsule(Vec3::ZERO, forearm_height, arm_radius))
        .insert(ImpulseJoint::new(upperarm_entity, forearm_joint))
        .insert(ActiveHooks::MODIFY_SOLVER_CONTACTS)
        .insert(ContactFilter::default())
        .insert(ConnectedEntities::default())
        .insert(ArmId(index))
        .insert(Muscle::new(forearm_target))
        .id();

    let mut hand_joint = SphericalJointBuilder::new()
        //.local_anchor2(Vec3::new(0.0, arm_radius + hand_radius, 0.0))
        .local_anchor2(Vec3::new(0.0, arm_radius, 0.0))
        .motor_model(JointAxis::AngX, motor_model)
        .motor_model(JointAxis::AngY, motor_model)
        .motor_model(JointAxis::AngZ, motor_model)
        .motor_max_force(JointAxis::AngX, max_force)
        .motor_max_force(JointAxis::AngY, max_force)
        .motor_max_force(JointAxis::AngZ, max_force)
        .motor_position(
            JointAxis::AngX,
            0.0,
            resting_stiffness * 2.0,
            resting_damping * 2.0,
        )
        .motor_position(
            JointAxis::AngZ,
            0.0,
            resting_stiffness * 2.0,
            resting_damping * 2.0,
        )
        //.motor_position(JointAxis::AngY, 0.0, twist_stiffness, twist_damping)
        .build();
    hand_joint.set_contacts_enabled(false);

    let _hand_entity = commands
        .spawn(TransformBundle::from_transform(to_transform))
        .insert(Name::new(format!("Hand {}", index)))
        .insert(Hand)
        .insert(ConnectedEntities::default())
        .insert(ConnectedMass::default())
        .insert(GrabbedEntities::default())
        .insert(Grabbing { ..default() })
        .insert(ExternalImpulse::default())
        .insert(Velocity::default())
        .insert(ReadMassProperties::default())
        .insert(RigidBody::Dynamic)
        .insert(crate::physics::REST_GROUPING)
        .insert(Collider::ball(hand_radius))
        .insert(ImpulseJoint::new(forearm_entity, hand_joint))
        .insert(ActiveHooks::MODIFY_SOLVER_CONTACTS)
        .insert(ContactFilter::default())
        //.insert(crate::Slottable) // kind of funny lol
        .insert(ArmId(index))
        .insert(Muscle::new(hand_target))
        .insert(MuscleIKTarget::new(target))
        .id();
}

/// Traverse the transform hierarchy and joint hierarchy to find all related entities.
pub fn connected_entities(
    names: Query<&Name>,
    mut related: Query<
        (Entity, &mut ConnectedEntities),
        /*
               Or<(
                   Changed<Children>,
                   Changed<Parent>,
                   Changed<ImpulseJoint>,
                   Changed<JointChildren>,
               )>,
        */
    >,
    childrens: Query<&Children>,
    parents: Query<&Parent>,
    joint_childrens: Query<&JointChildren>,
    joints: Query<&ImpulseJoint, Without<GrabJoint>>,
) {
    for (core_entity, mut related) in &mut related {
        let mut related_entities = HashSet::new();
        related_entities.insert(core_entity);

        let mut entity_stack = related_entities.clone();
        while entity_stack.len() > 0 {
            let mut new_stack = HashSet::new();
            for entity in entity_stack.iter() {
                if let Ok(parent) = parents.get(*entity) {
                    let entity = parent.get();
                    if related_entities.insert(entity) {
                        new_stack.insert(entity);
                    }
                }

                if let Ok(children) = childrens.get(*entity) {
                    for child in children {
                        let entity = *child;
                        if related_entities.insert(entity) {
                            new_stack.insert(entity);
                        }
                    }
                }

                if let Ok(joint) = joints.get(*entity) {
                    let entity = joint.parent;
                    if related_entities.insert(entity) {
                        new_stack.insert(entity);
                    }
                }

                if let Ok(joint_children) = joint_childrens.get(*entity) {
                    for child in &joint_children.0 {
                        let entity = *child;
                        if related_entities.insert(entity) {
                            new_stack.insert(entity);
                        }
                    }
                }
            }

            entity_stack = new_stack;
        }

        let mut named = Vec::new();
        for entity in &related_entities {
            named.push(match names.get(*entity) {
                Ok(name) => name.as_str().to_owned(),
                _ => format!("{:?}", entity),
            });
        }

        **related = related_entities;
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct ConnectedMass(pub f32);

impl Default for ConnectedMass {
    fn default() -> Self {
        Self(0.0)
    }
}

pub fn connected_mass(
    mut connected: Query<(Entity, &mut ConnectedMass, &ConnectedEntities)>,
    masses: Query<&ReadMassProperties>,
    //names: Query<&Name>,
) {
    for (entity, mut connected_mass, connected) in &mut connected {
        let mut summed_mass = 0.0;
        for attached in &connected.grabbed {
            if let Ok(_part_mass) = masses.get(*attached) {
                //summed_mass += part_mass.0.mass;
            }
        }

        if let Ok(part_mass) = masses.get(entity) {
            summed_mass += part_mass.0.mass;
        }
        connected_mass.0 = summed_mass * 1.0;
        //info!("{:?} summed mass: {:?}", names.named(entity), summed_mass);
    }
}

pub fn contact_filter(mut connected: Query<(Entity, &mut ContactFilter, &ConnectedEntities)>) {
    for (_entity, mut contact_filter, connected) in &mut connected {
        contact_filter.0 = connected.grabbed.clone();
    }
}

pub fn extended_mass(mut command: Commands, masses: Query<(Entity, &ConnectedMass)>) {
    for (entity, mass) in &masses {
        command
            .entity(entity)
            .insert(springy::rapier::ExtendedMass(mass.0));
    }
}

pub fn ease_sine(x: f32) -> f32 {
    -((PI * x).cos() - 1.0) / 2.0
}

pub fn setup_ik(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    added_query: Query<(Entity, &Parent), Added<AnimationPlayer>>,
    children: Query<(Option<&JointChildren>, Option<&Children>)>,
    names: Query<&Name>,
    parents: Query<&Parent>,
) {
    for (entity, parent) in added_query.iter() {
        let player = parents.get(parent.get()).unwrap().get();

        info!(
            "added animation player to {:?}, {:?}",
            names.named(entity),
            names.named(player),
        );
        let mesh_right_hand = if let Ok(found_entity) = find_entity(
            &EntityPath {
                parts: vec![
                    "Pelvis".into(),
                    "Spine1".into(),
                    "Spine2".into(),
                    "Collar.R".into(),
                    "UpperArm.R".into(),
                    "ForeArm.R".into(),
                    "Hand.R".into(),
                ],
            },
            entity,
            &children,
            &names,
        ) {
            found_entity
        } else {
            continue;
        };

        let physics_right_hand = if let Ok(found_entity) = find_entity(
            &EntityPath {
                parts: vec!["Arm 0".into(), "Hand 0".into()],
            },
            player,
            &children,
            &names,
        ) {
            found_entity
        } else {
            continue;
        };

        let mesh_left_hand = if let Ok(found_entity) = find_entity(
            &EntityPath {
                parts: vec![
                    "Pelvis".into(),
                    "Spine1".into(),
                    "Spine2".into(),
                    "Collar.L".into(),
                    "UpperArm.L".into(),
                    "ForeArm.L".into(),
                    "Hand.L".into(),
                ],
            },
            entity,
            &children,
            &names,
        ) {
            found_entity
        } else {
            continue;
        };

        let physics_left_hand = if let Ok(found_entity) = find_entity(
            &EntityPath {
                parts: vec!["Arm 1".into(), "Hand 1".into()],
            },
            player,
            &children,
            &names,
        ) {
            found_entity
        } else {
            continue;
        };

        info!("entities: {:?}, {:?}", mesh_right_hand, physics_right_hand);

        let pole_target = commands
            .spawn(PbrBundle {
                transform: Transform::from_xyz(-1.0, 0.4, -0.4),
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.05,
                    subdivisions: 1,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::GREEN,
                    ..default()
                }),
                ..default()
            })
            .id();

        // Add an IK constraint to the right hand, using the targets that were created earlier.
        commands.entity(mesh_right_hand).insert(IkConstraint {
            chain_length: 2,
            iterations: 20,
            target: physics_right_hand,
            pole_target: Some(pole_target),
            pole_angle: std::f32::consts::FRAC_PI_2,
        });

        let pole_target = commands
            .spawn(PbrBundle {
                transform: Transform::from_xyz(1.0, 0.4, -0.4),
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.05,
                    subdivisions: 1,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::GREEN,
                    ..default()
                }),
                ..default()
            })
            .id();

        commands.entity(mesh_left_hand).insert(IkConstraint {
            chain_length: 2,
            iterations: 20,
            target: physics_left_hand,
            pole_target: Some(pole_target),
            pole_angle: std::f32::consts::FRAC_PI_2,
        });

        commands.entity(entity).insert(Transform {
            rotation: Quat::from_axis_angle(Vec3::Y, PI),
            //scale: Vec3::new(2.0, 1.5, 1.0),
            scale: Vec3::splat(1.0),
            //translation: Vec3::new(0.0, -1.0, 0.0),
            translation: Vec3::new(0.0, 0.0, 0.0),
        });
    }
}

fn find_entity(
    path: &EntityPath,
    root: Entity,
    children: &Query<(Option<&JointChildren>, Option<&Children>)>,
    names: &Query<&Name>,
) -> Result<Entity, ()> {
    let mut current_entity = root;

    for part in path.parts.iter() {
        let mut found = false;
        if let Ok((joint_children, children)) = children.get(current_entity) {
            if let Some(children) = children {
                for child in children.iter() {
                    if let Ok(name) = names.get(*child) {
                        if name == part {
                            current_entity = *child;
                            found = true;
                            break;
                        }
                    }
                }
            }

            if let Some(children) = joint_children {
                for child in children.iter() {
                    if let Ok(name) = names.get(*child) {
                        if name == part {
                            current_entity = *child;
                            found = true;
                            break;
                        }
                    }
                }
            }
        }

        if !found {
            warn!("Entity not found for path {:?} on part {:?}", path, part);
            return Err(());
        }
    }

    Ok(current_entity)
}
