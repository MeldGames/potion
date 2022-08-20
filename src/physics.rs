/// Collision Grouping Flags
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use sabi::stage::{NetworkCoreStage, NetworkSimulationAppExt};

bitflags::bitflags! {
    pub struct Group: u32 {
        const PLAYER = 1 << 0;
        const TERRAIN = 1 << 1;
        const ABILITY = 1 << 2;
        const FLUFF = 1 << 3;

        const PLAYER_FILTER = Group::PLAYER.bits() | Group::TERRAIN.bits();
        const TERRAIN_FILTER = Group::PLAYER.bits() | Group::TERRAIN.bits() | Group::FLUFF.bits();
        const ABILITY_FILTER = 0;
    }
}

pub const PLAYER_GROUPING: CollisionGroups =
    CollisionGroups::new(Group::PLAYER.bits(), Group::PLAYER_FILTER.bits());
pub const TERRAIN_GROUPING: CollisionGroups =
    CollisionGroups::new(Group::TERRAIN.bits(), Group::TERRAIN_FILTER.bits());
pub const ABILITY_GROUPING: CollisionGroups =
    CollisionGroups::new(Group::ABILITY.bits(), Group::ABILITY_FILTER.bits());

pub fn modify_rapier_context(mut context: ResMut<RapierContext>) {
    // Try to avoid launching players in weird situations
    context.integration_parameters.max_penetration_correction = 100.0;
    //context.integration_parameters.dt = crate::network::TICK_RATE.as_secs_f32();
    //info!("integration: {:?}", context.integration_parameters);
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::Fixed {
                dt: crate::network::TICK_RATE.as_secs_f32(),
                substeps: 1,
            },
            ..Default::default()
        });

        let physics_plugin =
            RapierPhysicsPlugin::<NoUserData>::default().with_default_system_setup(false);
        app.add_plugin(physics_plugin);

        app.add_network_stage_after(
            NetworkCoreStage::Update,
            PhysicsStages::SyncBackend,
            SystemStage::parallel().with_system_set(
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsStages::SyncBackend),
            ),
        );
        app.add_network_stage_after(
            PhysicsStages::SyncBackend,
            PhysicsStages::StepSimulation,
            SystemStage::parallel().with_system_set(
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsStages::StepSimulation),
            ),
        );
        app.add_network_stage_after(
            PhysicsStages::StepSimulation,
            PhysicsStages::Writeback,
            SystemStage::parallel().with_system_set(
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsStages::Writeback),
            ),
        );

        // NOTE: we run sync_removals at the end of the frame, too, in order to make sure we don’t miss any `RemovedComponents`.
        app.add_network_stage_before(
            NetworkCoreStage::Last,
            PhysicsStages::DetectDespawn,
            SystemStage::parallel().with_system_set(
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsStages::DetectDespawn),
            ),
        );

        app.add_startup_system(modify_rapier_context);
    }
}
