
use crate::prelude::*;


pub struct PotionPlugin;

impl Plugin for PotionPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<Potion>()
        .register_type::<CrackThreshold>();

    app.add_system(potion_contact_explode);
    }
}

#[derive(Component, Debug, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct Potion;

#[derive(Component, Debug, Reflect, FromReflect)]
#[reflect(Component)]
pub struct CrackThreshold(f32);

impl Default for CrackThreshold {
    fn default() -> Self {
        Self(0.1)
    }
}

#[derive(Bundle)]
pub struct PotionBundle {
    pub potion: Potion,
    pub crack_threshold: CrackThreshold,
    pub contact_force_event_threshold: ContactForceEventThreshold,
}

impl Default for PotionBundle {
    fn default() -> Self {
        Self {
            potion: Potion::default(),
            crack_threshold: CrackThreshold::default(),
            contact_force_event_threshold: ContactForceEventThreshold(0.0),
        }
    }
}

pub fn potion_contact_explode(mut commands: Commands, potions: Query<(&Potion, &CrackThreshold)>, mut contact_forces: EventReader<ContactForceEvent>) {
    let mut check_crack = |entity: Entity, event: &ContactForceEvent| -> bool {
        info!("checking crack: {:?}", entity);
        let Ok((potion, crack_threshold)) = potions.get(entity) else { return false };
        let hit_force = event.max_force_magnitude.abs();
        let cracked = hit_force > crack_threshold.0;
        if cracked {
            info!("entity {:?} cracked at force {:?}", entity, hit_force);
            commands.entity(entity).despawn_recursive();
        }

        cracked
    };

    for contact_force in contact_forces.iter() {
        check_crack(contact_force.collider1, &contact_force);
        check_crack(contact_force.collider2, &contact_force);
    }
}