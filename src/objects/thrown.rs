
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct ThrowPlugin;
impl Plugin for ThrowPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Thrown>();
        app.add_systems(FixedUpdate, (throw_decay,));
    }
}


#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct Thrown {
    /// Time until this component is removed.
    pub timer: f32,
}

impl Default for Thrown {
    fn default() -> Self {
        Self {
            timer: 1.0,
        }
    }
}

pub fn throw_decay(mut commands: Commands, ctx: Res<RapierContext>, mut thrown: Query<(Entity, &mut Thrown, Option<&Velocity>)>) {
    let dt = ctx.integration_parameters.dt;
    for (entity, mut thrown, velocity) in &mut thrown {
        thrown.timer -= dt;

        let mut remove = false;
        if let Some(velocity) = velocity {
            if velocity.linvel.length() < 0.1 {
                remove = true;
            }
        }

        if thrown.timer <= 0.0 {
            remove = true;
        }

        if remove {
            commands.entity(entity).remove::<Thrown>();
        }
    }
}