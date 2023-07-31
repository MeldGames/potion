use bevy::{prelude::*, transform::TransformSystem};
use bevy_rapier3d::prelude::*;

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

pub fn ground_truth_transform(
    mut entity: Entity,
    transforms: &Query<&Transform>,
    parents: &Query<&Parent>,
) -> Transform {
    let mut transform = transforms
        .get(entity)
        .cloned()
        .unwrap_or(Transform::default());
    while let Ok(parent) = parents.get(entity) {
        if let Ok(parent_transform) = transforms.get(parent.get()) {
            transform = *parent_transform * transform;
        }

        entity = parent.get()
    }

    transform
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
    mut transforms: Query<&mut Transform>,
    parents: Query<&Parent>,
) {
    for (attach_entity, attach, translation, rotation, scale) in &mut attachers {
        let particle_entity = attach.get();
        let ground_truth =
            ground_truth_transform(particle_entity, &transforms.to_readonly(), &parents);

        if let Ok(mut transform) = transforms.get_mut(attach_entity) {
            match translation {
                Some(AttachTranslation::Instant) => {
                    transform.translation = ground_truth.translation;
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

                    let instant = particle_a.translation().instant(&particle_b.translation());
                    let impulse = spring.impulse(timestep, instant);

                    let [attach_impulse, particle_impulse] = if let Ok(impulses) =
                        impulses.get_many_mut([attach_entity, particle_entity])
                    {
                        impulses
                    } else {
                        warn!("Particle does not contain all necessary components");
                        continue;
                    };

                    if let Some(mut attach_impulse) = attach_impulse {
                        attach_impulse.impulse += -impulse;
                    }

                    if let Some(mut particle_impulse) = particle_impulse {
                        particle_impulse.impulse += impulse;
                    }
                }
                _ => {}
            }

            match rotation {
                Some(AttachRotation::Instant) => {
                    transform.rotation = ground_truth.rotation;
                    //velocity.angvel = Vec3::ZERO;
                }
                Some(AttachRotation::Inverse) => {
                    transform.rotation = ground_truth.rotation.inverse();
                }
                _ => {}
            }

            if scale.is_some() {
                transform.scale = ground_truth.scale;
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

        app.add_systems(Update, velocity_nonphysics.in_set(crate::FixedSet::Update));

        app.add_systems(
            FixedUpdate,
            (
                update_attach.before(PhysicsSet::SyncBackend),
                update_attach.after(PhysicsSet::Writeback),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_no_lag() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .add_plugins(HierarchyPlugin)
            .add_plugins(TransformPlugin)
            .add_plugins(AttachPlugin);

        fn check_globals(
            attached: Query<(&Attach, &GlobalTransform)>,
            globals: Query<&GlobalTransform>,
        ) {
            for (attach, global) in &attached {
                if let Ok(other_global) = globals.get(attach.get()) {
                    println!(
                        "{:.2} == {:.2}",
                        global.translation(),
                        other_global.translation()
                    );
                    assert_eq!(global.translation(), other_global.translation())
                }
            }
        }

        app.add_systems(Last, check_globals);

        let core = app
            .world
            .spawn(SpatialBundle {
                transform: Transform::from_xyz(5.0, 0.0, 0.0),
                ..default()
            })
            .id();

        app.world
            .spawn(SpatialBundle::default())
            .insert(Attach::translation(core));

        app.update();
        app.update();
    }
}
