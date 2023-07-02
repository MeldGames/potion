use std::fmt::Debug;

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_debug_lines::DebugLines;
use std::f32::consts::PI;

use bevy_mod_wanderlust::{ControllerInput, ControllerSettings};
use bevy_rapier3d::prelude::*;

use super::input::PlayerInput;
use crate::prelude::*;

pub struct ControllerPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControllerSet;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                player_movement,
                avoid_intersecting,
                character_crouch,
                controller_exclude,
                player_swivel_and_tilt,
                teleport_player_back,
            )
                .in_set(ControllerSet)
                .before(bevy_mod_wanderlust::movement)
                .in_set(crate::FixedSet::Update)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// Entities that should be considered as part of the controlled character, not including grabbed.
#[derive(Deref, DerefMut, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct CharacterEntities(HashSet<Entity>);

/// Entities that should be considered connected to this entity in some way.
#[derive(Deref, DerefMut, Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct ConnectedEntities(HashSet<Entity>);

pub fn player_movement(
    mut query: Query<
        (
            &GlobalTransform,
            &mut ControllerInput,
            &mut ExternalImpulse,
            &LookTransform,
            &PlayerInput,
        ),
        //With<Owned>,
    >,
    _lines: ResMut<DebugLines>,
) {
    for (global, mut controller, mut impulse, _look_transform, player_input) in query.iter_mut() {
        let mut dir = Vec3::new(0.0, 0.0, 0.0);
        if player_input.left() {
            dir.x += -1.;
        }
        if player_input.right() {
            dir.x += 1.;
        }

        if player_input.back() {
            dir.z += 1.;
        }
        if player_input.forward() {
            dir.z += -1.;
        }

        // we only take into account horizontal rotation so looking down doesn't
        // slow the character down.
        let rotation = Quat::from_axis_angle(Vec3::Y, player_input.yaw as f32);
        let dir = (rotation * dir).normalize_or_zero();

        controller.movement = dir;
        controller.jumping = player_input.jump();

        let current_dir = Vec2::new(global.forward().x, global.forward().z);
        //let mut desired_dir = Vec2::new(dir.x, dir.z);

        // If we are grabby then make the character face the way we are grabbing.
        //if player_input.any_extend_arm() {
        let camera_dir = rotation * -Vec3::Z;
        let desired_dir = Vec2::new(camera_dir.x, camera_dir.z);
        //}

        let _spring = springy::Spring {
            strength: 0.1,
            damp_ratio: 1.0,
        };

        if desired_dir.length() > 0.0 && current_dir.length() > 0.0 {
            let y = desired_dir.angle_between(current_dir);
            impulse.torque_impulse.y += y * 0.5;
        }
    }
}

pub fn teleport_player_back(
    mut players: Query<Entity, With<Player>>,
    kb: Res<Input<KeyCode>>,
    _names: Query<&Name>,

    _parents: Query<&Parent>,
    children: Query<&Children>,
    joint_children: Query<&JointChildren>,
    related: Query<Entity>,

    mut velocities: Query<&mut Velocity, With<RigidBody>>,
    mut transforms: Query<&mut Transform>,
) {
    for entity in &mut players {
        let mut should_teleport = kb.just_pressed(KeyCode::Equals);

        if let Ok(transform) = transforms.get(entity) {
            should_teleport = should_teleport || transform.translation.y < -100.0;
            should_teleport = should_teleport || transform.translation.y > 1000.0;
            should_teleport = should_teleport || transform.translation.x < -1000.0;
            should_teleport = should_teleport || transform.translation.x > 1000.0;
            should_teleport = should_teleport || transform.translation.z < -1000.0;
            should_teleport = should_teleport || transform.translation.z > 1000.0;
        }

        if should_teleport {
            let results = find_children_with(&related, &children, &joint_children, entity);

            let mut relative_positions = bevy::utils::HashMap::new();
            for result in results {
                /*
                let debug_name = names.get(result)
                    .map(|name| name.as_str().to_owned())
                    .unwrap_or(format!("{:?}", result));
                info!("resetting velocity: {:?}", debug_name);
                */

                if let Ok(mut velocity) = velocities.get_mut(result) {
                    velocity.linvel = Vec3::ZERO;
                    velocity.angvel = Vec3::ZERO;

                    if let Ok([transform, other]) = transforms.get_many([entity, result]) {
                        let relative = transform.translation - other.translation;
                        relative_positions.insert(result, relative);
                    }
                }
            }

            let new_position = Vec3::new(0.0, 10.0, 0.0);
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = new_position;
                transform.rotation = Quat::IDENTITY;
            }

            for (entity, relative_position) in relative_positions {
                if let Ok(mut transform) = transforms.get_mut(entity) {
                    transform.translation = new_position + relative_position;
                }
            }
        }
    }
}

pub fn character_crouch(mut controllers: Query<(&PlayerInput, &mut ControllerSettings)>) {
    let crouch_height = 0.15;
    let full_height = 1.0;
    let threshold = -PI / 4.0;
    for (input, mut controller) in &mut controllers {
        // Are we looking sufficiently down?
        if input.pitch < threshold {
            // interpolate between crouch and full based on how far we are pitched downwards
            let crouch_coefficient =
                (input.pitch.abs() - threshold.abs()) / ((PI / 2.0) - threshold.abs());
            let interpolated =
                full_height * (1.0 - crouch_coefficient) + crouch_height * crouch_coefficient;
            controller.float_distance = interpolated;
        } else {
            controller.float_distance = full_height;
        }
    }
}

pub fn controller_exclude(
    _names: Query<&Name>,
    mut controllers: Query<(Entity, Option<&CharacterEntities>, &mut ControllerSettings)>,
) {
    for (_entity, connected, mut settings) in &mut controllers {
        let mut new_exclude = HashSet::new();

        if let Some(connected) = connected {
            new_exclude.extend(connected.iter());
        }

        settings.exclude_from_ground = new_exclude;
    }
}

pub fn player_swivel_and_tilt(
    mut inputs: Query<(&mut LookTransform, &PlayerInput, &PlayerNeck)>,
    mut necks: Query<&mut Transform, (With<Neck>, Without<Player>)>,
) {
    for (mut look_transform, input, neck) in &mut inputs {
        if let Ok(mut neck_transform) = necks.get_mut(neck.0) {
            let rotation = (Quat::from_axis_angle(Vec3::Y, input.yaw as f32)
                * Quat::from_axis_angle(Vec3::X, input.pitch as f32))
            .into();

            neck_transform.rotation = rotation;
            look_transform.0 = *neck_transform;
        }
    }
}

#[derive(Default, Debug, Clone, Component)]
pub struct LookTransform(pub Transform);

impl LookTransform {
    pub fn rotation(&self) -> Quat {
        self.0.rotation
    }

    pub fn translation(&self) -> Vec3 {
        self.0.translation
    }
}

#[derive(Component, Debug, Clone)]
pub struct AvoidIntersecting {
    pub dir: Vec3,
    pub max_toi: f32,
    pub buffer: f32,
}

pub fn avoid_intersecting(
    rapier_context: Res<RapierContext>,
    global: Query<&GlobalTransform>,
    mut avoid: Query<(&mut Transform, &Parent, &AvoidIntersecting)>,
) {
    let filter = QueryFilter::exclude_dynamic().exclude_sensors();

    for (mut transform, parent, avoid) in &mut avoid {
        let global_transform = if let Ok(global) = global.get(parent.get()) {
            global.compute_transform()
        } else {
            Transform::default()
        };

        let (toi, normal) = if let Some((_entity, intersection)) = rapier_context
            .cast_ray_and_get_normal(
                global_transform.translation,
                global_transform.rotation * avoid.dir,
                avoid.max_toi + avoid.buffer,
                true,
                filter,
            ) {
            (intersection.toi, intersection.normal)
        } else {
            (avoid.max_toi + avoid.buffer, Vec3::ZERO)
        };

        transform.translation = avoid.dir * toi + (normal * avoid.buffer);
    }
}
