use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use sabi::stage::NetworkSimulationAppExt;

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct BreakableJoint {
    pub impulse: Vec3,
    pub torque: Vec3,
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct BreakGracePeriod(pub f32);

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct BreakPastGracePeriod;

pub fn break_grace_period(
    mut commands: Commands,
    //time: Res<Time>,
    breakable: Query<(Entity, &BreakableJoint)>,
    mut grace: Query<&mut BreakGracePeriod>,
) {
    for (entity, _break_at) in &breakable {
        match grace.get_mut(entity) {
            Ok(mut grace) => {
                grace.0 -= crate::TICK_RATE.as_secs_f32();
                if grace.0 <= 0.0 {
                    commands
                        .entity(entity)
                        .remove::<BreakGracePeriod>()
                        .insert(BreakPastGracePeriod);
                }
            }
            _ => {
                commands.entity(entity).insert(BreakGracePeriod(2.0));
            }
        }
    }
}

pub fn break_joints(
    mut commands: Commands,
    rapier_ctx: Res<RapierContext>,
    breakable: Query<
        (Entity, &RapierImpulseJointHandle, &BreakableJoint),
        With<BreakPastGracePeriod>,
    >,
) {
    for (entity, joint, break_at) in &breakable {
        if let Some(joint) = rapier_ctx.impulse_joints.get(joint.0) {
            let impulses = joint.impulses;
            let impulse = Vec3::new(impulses.x, impulses.y, impulses.z);
            let torque = Vec3::new(impulses.w, impulses.a, impulses.b);

            if impulse.x >= break_at.impulse.x
                || impulse.y >= break_at.impulse.y
                || impulse.z >= break_at.impulse.z
                || torque.x >= break_at.torque.x
                || torque.y >= break_at.torque.y
                || torque.z >= break_at.torque.z
            {
                info!("broke at: i {:.2?} t {:.2}", impulse, torque);
                commands.entity(entity).remove::<ImpulseJoint>();
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BreakJointPlugin;

impl Plugin for BreakJointPlugin {
    fn build(&self, app: &mut App) {
        app.add_network_system(
            break_grace_period
                .label("brake_grace_period")
                .after("break_joints"),
        );
        app.add_network_system(break_joints.label("break_joints"));
    }
}
