use bevy::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct Follow(pub Entity);

impl Follow {
    pub fn entity(&self) -> Entity {
        self.0
    }
}

pub fn update_follow(
    mut followers: Query<(&mut Transform, &Follow)>,
    global: Query<&GlobalTransform>,
) {
    for (mut transform, follow) in &mut followers {
        if let Ok(global) = global.get(follow.entity()) {
            transform.translation = global.translation();
        }
    }
}

pub struct FollowPlugin;

impl Plugin for FollowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_follow);
    }
}
