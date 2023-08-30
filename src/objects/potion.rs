use crate::objects::Thrown;
use crate::prelude::*;

pub struct PotionPlugin;

impl Plugin for PotionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Potion>()
            .register_type::<CrackThreshold>();

        app.add_systems(FixedUpdate, (potion_contact_explode,));
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct Potion;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CrackThreshold(f32);

impl Default for CrackThreshold {
    fn default() -> Self {
        Self(200.0)
    }
}

#[derive(Bundle)]
pub struct PotionBundle {
    pub potion: Potion,
    pub crack_threshold: CrackThreshold,
}

impl Default for PotionBundle {
    fn default() -> Self {
        Self {
            potion: Potion::default(),
            crack_threshold: CrackThreshold::default(),
        }
    }
}

#[derive(Bundle)]
pub struct PotionColliderBundle {
    pub contact_force_event_threshold: ContactForceEventThreshold,
    pub active_events: ActiveEvents,
}

impl Default for PotionColliderBundle {
    fn default() -> Self {
        Self {
            contact_force_event_threshold: ContactForceEventThreshold(5.0),
            active_events: ActiveEvents::CONTACT_FORCE_EVENTS,
        }
    }
}

pub fn potion_contact_explode(
    mut commands: Commands,
    potions: Query<&CrackThreshold, (With<Potion>, With<Thrown>)>,
    globals: Query<&GlobalTransform>,
    velocities: Query<&Velocity>,
    mut contact_forces: EventReader<ContactForceEvent>,
    rigid_body: Query<Entity, With<RigidBody>>,
    parent: Query<&Parent>,
    mut gizmos: ResMut<RetainedGizmos>,
) {
    let mut check_crack = |mut entity: Entity, other: Entity, event: &ContactForceEvent| -> bool {
        while !rigid_body.contains(entity) {
            if let Ok(parent) = parent.get(entity) {
                entity = parent.get();
            } else {
                return false;
            }
        }

        let Ok(crack_threshold) = potions.get(entity) else {
            return false;
        };
        let hit_force = event.max_force_magnitude.abs();
        let cracked = hit_force > crack_threshold.0;
        if cracked {
            info!("entity {:?} cracked at force {:?}", entity, hit_force);
            commands.entity(entity).despawn_recursive();
            let global = globals
                .get(entity)
                .cloned()
                .unwrap_or(GlobalTransform::IDENTITY);
            let velocity = velocities
                .get(entity)
                .cloned()
                .unwrap_or(Velocity::default());

            commands
                .spawn(SpatialBundle {
                    transform: global.compute_transform(),
                    ..default()
                })
                .insert(crate::objects::EffectVelocity {
                    //linear: velocity.linvel,
                    linear: -event.total_force,
                })
                .insert(crate::objects::vine::VineEffect);

            gizmos.sphere(
                4.0,
                global.translation(),
                Quat::IDENTITY,
                3.0,
                Color::PURPLE,
            );
        }

        cracked
    };

    for contact_force in contact_forces.iter() {
        check_crack(
            contact_force.collider1,
            contact_force.collider2,
            &contact_force,
        );
        check_crack(
            contact_force.collider2,
            contact_force.collider1,
            &contact_force,
        );
    }
}
