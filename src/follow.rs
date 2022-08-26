use bevy::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct Follow(Entity);

impl Follow {
    pub fn scale(entity: Entity) -> (Follow, FollowScale) {
        (Follow(entity), FollowScale)
    }

    pub fn rotation(entity: Entity) -> (Follow, FollowRotation) {
        (Follow(entity), FollowRotation)
    }

    pub fn translation(entity: Entity) -> (Follow, FollowTranslation) {
        (Follow(entity), FollowTranslation)
    }

    pub fn all(entity: Entity) -> (Follow, FollowTranslation, FollowRotation, FollowScale) {
        (
            Follow(entity),
            FollowTranslation,
            FollowRotation,
            FollowScale,
        )
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Clone, Component)]
pub struct FollowTranslation;

#[derive(Debug, Clone, Component)]
pub struct FollowRotation;

#[derive(Debug, Clone, Component)]
pub struct FollowScale;

pub fn update_follow(
    mut followers: Query<
        (
            &mut Transform,
            &Follow,
            Option<&FollowTranslation>,
            Option<&FollowRotation>,
            Option<&FollowScale>,
        ),
        Or<(
            With<FollowTranslation>,
            With<FollowRotation>,
            With<FollowScale>,
        )>,
    >,
    global: Query<&GlobalTransform>,
) {
    for (mut transform, follow, translation, rotation, scale) in &mut followers {
        if let Ok(global) = global.get(follow.get()) {
            let global_transform = global.compute_transform();
            if translation.is_some() {
                transform.translation = global_transform.translation;
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

pub struct FollowPlugin;

impl Plugin for FollowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_follow);
    }
}
