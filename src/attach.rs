use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLines;
use sabi::stage::{NetworkCoreStage, NetworkSimulationAppExt};

#[derive(Debug, Clone, Component)]
pub struct Attach(Entity);

impl Attach {
    pub fn scale(entity: Entity) -> (Attach, AttachScale) {
        (Attach(entity), AttachScale::Instant)
    }

    pub fn rotation(entity: Entity) -> (Attach, AttachRotation) {
        (Attach(entity), AttachRotation::Instant)
    }

    pub fn translation(entity: Entity) -> (Attach, AttachTranslation) {
        (Attach(entity), AttachTranslation::Instant)
    }

    pub fn all(entity: Entity) -> (Attach, AttachTranslation, AttachRotation, AttachScale) {
        (
            Attach(entity),
            AttachTranslation::Instant,
            AttachRotation::Instant,
            AttachScale::Instant,
        )
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Default, Debug, Clone, Component, Reflect, Inspectable)]
#[reflect(Component)]
pub enum AttachTranslation {
    #[default]
    Instant,
    Spring {
        #[inspectable(min = 0.0, max = 10000.0)]
        strength: f32,
        #[inspectable(min = 0.0, max = 1.0)]
        damp_ratio: f32,
    },
}

#[derive(Default, Debug, Clone, Component, Reflect, Inspectable)]
#[reflect(Component)]
pub enum AttachRotation {
    #[default]
    Instant,
    Spring {
        strength: f32,
        damp_ratio: f32,
    },
}

#[derive(Default, Debug, Clone, Component, Reflect, Inspectable)]
#[reflect(Component)]
pub enum AttachScale {
    #[default]
    Instant,
    Spring {
        strength: f32,
        damp_ratio: f32,
    },
}

#[derive(Debug, Clone, Component)]
pub struct PreviousTransform(pub Transform);

pub fn velocity_nonphysics(
    mut velocities: Query<(&mut Transform, &Velocity, &ExternalForce), Without<RigidBody>>,
) {
    for (mut position, velocity, accel) in &mut velocities {
        position.translation += velocity.linvel * crate::TICK_RATE.as_secs_f32();
    }
}

pub fn update_attach(
    time: Res<Time>,
    mut commands: Commands,
    parented: Query<Entity, (With<Attach>, With<Parent>)>,
    no_velocity: Query<Entity, (With<Attach>, Without<Velocity>)>,
    mut attachers: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            Option<&mut ExternalForce>,
            Option<&mut ExternalImpulse>,
            Option<&ReadMassProperties>,
            &Attach,
            Option<&AttachTranslation>,
            Option<&AttachRotation>,
            Option<&AttachScale>,
        ),
        Or<(
            With<AttachTranslation>,
            With<AttachRotation>,
            With<AttachScale>,
        )>,
    >,
    global: Query<&GlobalTransform>,
    names: Query<&Name>,
    mut lines: ResMut<DebugLines>,
) {
    let dt = time.delta_seconds();

    if dt == 0.0 {
        return;
    }

    let named = |entity: Entity| -> String {
        match names.get(entity) {
            Ok(name) => name.as_str().to_owned(),
            _ => format!("{:?}", entity),
        }
    };

    for invalid_attacher in &parented {
        info!(
            "attacher is invalid, cannot use the transform hierarchy: {:?}",
            named(invalid_attacher)
        );
    }

    for invalid_attacher in &no_velocity {
        info!(
            "attacher needs Velocity, adding default: {:?}",
            named(invalid_attacher)
        );

        commands
            .entity(invalid_attacher)
            .insert(Velocity::default());
    }

    for (
        entity,
        mut transform,
        mut velocity,
        mut force,
        mut impulse,
        mass_properties,
        attach,
        translation,
        rotation,
        scale,
    ) in &mut attachers
    {
        //info!("attaching {:?}", named(entity));
        if let Ok(global) = global.get(attach.get()) {
            //info!("to {:?}", named(attach.get()));
            let global_transform = global.compute_transform();
            match translation {
                Some(AttachTranslation::Instant) => {
                    transform.translation = global_transform.translation;
                }
                Some(&AttachTranslation::Spring {
                    strength,
                    damp_ratio,
                }) => {
                    let strength = strength.max(0.0);
                    let damp_ratio = damp_ratio.max(0.0);
                    let (mass, center) = match mass_properties {
                        Some(mass_properties) => (
                            mass_properties.0.mass,
                            mass_properties.0.local_center_of_mass,
                        ),
                        None => (1.0, Vec3::ZERO),
                    };

                    if mass <= 0.0 || strength <= 0.0 {
                        continue;
                    }

                    let critical_damping = 2.0 * (mass * strength).sqrt();
                    let damp_coefficient = damp_ratio * critical_damping;

                    let offset = transform.translation - global_transform.translation;
                    let offset_force = -strength * offset;
                    let vel =
                        velocity.linvel + velocity.angvel.cross(Vec3::ZERO - center) + offset_force;

                    let mut damp_force = -damp_coefficient * vel;

                    // don't let the damping force accelerate it
                    damp_force = damp_force.clamp_length_max(vel.length());

                    let spring_force = offset_force + damp_force;
                    //spring_force = spring_force.clamp_length_max(vel.length());

                    match impulse {
                        Some(mut impulse) => {
                            //external_force.force = spring_force;
                            impulse.impulse = spring_force * dt;
                        }
                        None => {
                            velocity.linvel += spring_force * dt;
                        }
                    }

                    //info!("length: {:?}", spring_force.length() / strength);
                    let lightness = (spring_force.length() / strength).clamp(0.0, 1.0);
                    let color = Color::Hsla {
                        hue: 0.0,
                        saturation: 1.0,
                        lightness: lightness,
                        alpha: 0.7,
                    };

                    lines.line_colored(
                        transform.translation,
                        transform.translation + spring_force,
                        crate::TICK_RATE.as_secs_f32(),
                        //Color::YELLOW,
                        color,
                    );
                }
                _ => {}
            }

            if rotation.is_some() {
                transform.rotation = global_transform.rotation;
                velocity.angvel = Vec3::ZERO;
            }

            if scale.is_some() {
                transform.scale = global_transform.scale;
            }
        }
    }
}

pub struct AttachPlugin;

impl Plugin for AttachPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttachTranslation>()
            .register_type::<AttachRotation>()
            .register_type::<AttachScale>();

        app.register_inspectable::<AttachTranslation>()
            .register_inspectable::<AttachRotation>()
            .register_inspectable::<AttachScale>();

        app.add_network_system(velocity_nonphysics.label("velocity_nonphysics"));
        app.add_network_system(update_attach.label("update_attach"));
    }
}
