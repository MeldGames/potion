use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::cauldron::NamedEntity;

#[derive(Debug, Clone, Component)]
pub struct Attach(Entity);

impl Attach {
    pub fn new(entity: Entity) -> Attach {
        Attach(entity)
    }

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

#[derive(Default, Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub enum AttachTranslation {
    #[default]
    Instant,
    Spring(springy::Spring),
}

#[derive(Default, Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub enum AttachRotation {
    #[default]
    Instant,
    Inverse,
    Spring(springy::Spring),
}

#[derive(Default, Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub enum AttachScale {
    #[default]
    Instant,
    Spring(springy::Spring),
}

#[derive(Debug, Clone, Component)]
pub struct PreviousTransform(pub Transform);

pub fn velocity_nonphysics(mut velocities: Query<(&mut Transform, &Velocity), Without<RigidBody>>) {
    for (mut position, velocity) in &mut velocities {
        position.translation += velocity.linvel * crate::TICK_RATE.as_secs_f32();
    }
}

pub fn update_attach(
    mut commands: Commands,
    //parented: Query<Entity, (With<Attach>, With<Parent>)>,
    no_velocity: Query<Entity, (With<Attach>, Without<Velocity>)>,
    particles: Query<springy::RapierParticleQuery>,
    mut impulses: Query<Option<&mut ExternalImpulse>>,
    mut attachers: Query<
        (
            Entity,
            &mut Transform,
            &Velocity,
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
    globals: Query<&GlobalTransform>,
    names: Query<&Name>,
) {
    /*
       for invalid_attacher in &parented {
           info!(
               "attacher is invalid, cannot use the transform hierarchy: {:?}",
               named(invalid_attacher)
           );
       }
    */
    for invalid_attacher in &no_velocity {
        info!(
            "attacher needs Velocity, adding default: {:?}",
            names.named(invalid_attacher)
        );

        commands
            .entity(invalid_attacher)
            .insert(Velocity::default());
    }

    for (attach_entity, mut transform, _velocity, attach, translation, rotation, scale) in
        &mut attachers
    {
        let particle_entity = attach.get();
        if let Ok(global) = globals.get(particle_entity) {
            let global_transform = global.compute_transform();
            match translation {
                Some(AttachTranslation::Instant) => {
                    transform.translation = global_transform.translation;
                }
                Some(&AttachTranslation::Spring(spring)) => {
                    let timestep = crate::TICK_RATE.as_secs_f32();
                    let [particle_a, particle_b] =
                        if let Ok(particles) = particles.get_many([attach_entity, attach.get()]) {
                            particles
                        } else {
                            warn!("Particle does not contain all necessary components");
                            continue;
                        };

                    let (impulse, _) = spring.impulse(timestep, particle_a, particle_b, None);

                    let [attach_impulse, particle_impulse] = if let Ok(impulses) =
                        impulses.get_many_mut([attach_entity, particle_entity])
                    {
                        impulses
                    } else {
                        warn!("Particle does not contain all necessary components");
                        continue;
                    };

                    if let Some(mut attach_impulse) = attach_impulse {
                        attach_impulse.impulse = -impulse;
                    }

                    if let Some(mut particle_impulse) = particle_impulse {
                        particle_impulse.impulse = impulse;
                    }
                }
                _ => {}
            }

            match rotation {
                Some(AttachRotation::Instant) => {
                    transform.rotation = global_transform.rotation;
                    //velocity.angvel = Vec3::ZERO;
                }
                Some(AttachRotation::Inverse) => {
                    transform.rotation = global_transform.rotation.inverse();
                }
                _ => {}
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

        app.add_system(velocity_nonphysics);
        app.add_system(update_attach.in_base_set(CoreSet::PreUpdate));
        app.add_system(update_attach.in_base_set(CoreSet::Update));
        app.add_system(update_attach.in_schedule(CoreSchedule::FixedUpdate).after(crate::player::controller::player_movement));
        app.add_system(update_attach.in_base_set(CoreSet::PostUpdate));
        //app.add_system(update_attach.label("update_attach"));
    }
}
