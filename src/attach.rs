use bevy::prelude::*;
use sabi::stage::NetworkSimulationAppExt;

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

#[derive(Debug, Clone, Component)]
pub enum AttachTranslation {
    Instant,
    Spring { strength: f32, dampening: f32 },
}

#[derive(Debug, Clone, Component)]
pub enum AttachRotation {
    Instant,
    Spring { strength: f32, dampening: f32 },
}

#[derive(Debug, Clone, Component)]
pub enum AttachScale {
    Instant,
    Spring { strength: f32, dampening: f32 },
}

#[derive(Debug, Clone, Component)]
pub struct PreviousTransform(pub Transform);

pub fn update_attach(
    mut commands: Commands,
    invalid_attachers: Query<Entity, (With<Attach>, With<Parent>)>,
    mut attachers: Query<
        (
            Entity,
            &mut Transform,
            Option<&mut PreviousTransform>,
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
) {
    for invalid_attacher in &invalid_attachers {
        info!(
            "attacher is invalid, cannot use the transform hierarchy: {:?}",
            invalid_attacher
        );
    }

    for (entity, mut transform, mut prev, attach, translation, rotation, scale) in &mut attachers {
        let previous = prev
            .as_ref()
            .map(|prev| prev.0.clone())
            .unwrap_or(*transform);

        if let Some(ref mut prev) = prev {
            prev.0 = *transform;
        } else {
            commands
                .entity(entity)
                .insert(PreviousTransform(*transform));
        }

        if let Ok(global) = global.get(attach.get()) {
            let global_transform = global.compute_transform();
            match translation {
                Some(AttachTranslation::Instant) => {
                    transform.translation = global_transform.translation;
                }
                Some(&AttachTranslation::Spring {
                    strength,
                    dampening,
                }) => {
                    let velocity = previous.translation - transform.translation;
                    let offset = global_transform.translation - transform.translation;
                    let impulse = (offset * strength) - (velocity * dampening);
                    transform.translation += impulse;
                }
                _ => {}
            }

            if rotation.is_some() {
                transform.rotation = global_transform.rotation;
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
        app.add_network_system(update_attach.label("update_attach"));
    }
}
