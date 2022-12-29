/// Collision Grouping Flags
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use sabi::stage::{NetworkCoreStage, NetworkSimulationAppExt};

pub mod contact_filter;
pub mod spring;

pub use contact_filter::*;
pub use spring::Spring;

bitflags::bitflags! {
    pub struct Groups: u32 {
        const PLAYER = 1 << 0;
        const TERRAIN = 1 << 1;
        const FLUFF = 1 << 3;

        const PLAYER_FILTER = Groups::PLAYER.bits() | Groups::TERRAIN.bits();
        const TERRAIN_FILTER = Groups::PLAYER.bits() | Groups::TERRAIN.bits() | Groups::FLUFF.bits();
    }
}

pub const PLAYER_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::PLAYER.bits()),
    Group::from_bits_truncate(Groups::PLAYER_FILTER.bits()),
);

pub const TERRAIN_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::TERRAIN.bits()),
    Group::from_bits_truncate(Groups::TERRAIN_FILTER.bits()),
);

pub const REST_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::PLAYER.bits()),
    Group::from_bits_truncate(Groups::PLAYER.bits()),
);
pub const GRAB_GROUPING: CollisionGroups = PLAYER_GROUPING;

pub fn modify_rapier_context(mut context: ResMut<RapierContext>) {
    let integration = &mut context.integration_parameters;
    integration.damping_ratio = 0.5;
    integration.joint_erp = 0.8;
    integration.joint_damping_ratio = 0.5;
    // Try to avoid launching players in weird situations
    integration.max_penetration_correction = 1000.0;
    integration.dt = crate::TICK_RATE.as_secs_f32();
}

pub const VELOCITY_CAP: f32 = 1000.0;
pub const ANG_VELOCITY_CAP: f32 = 50.0;

pub fn cap_velocity(mut velocities: Query<&mut Velocity, Changed<Velocity>>) {
    for mut velocity in &mut velocities {
        velocity.linvel = velocity.linvel.clamp_length_max(VELOCITY_CAP);
        velocity.angvel = velocity.angvel.clamp_length_max(ANG_VELOCITY_CAP);
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::Fixed {
                dt: crate::TICK_RATE.as_secs_f32(),
                substeps: 1,
            },
            ..Default::default()
        });

        app.insert_resource(PhysicsHooksWithQueryResource::<HookData>(Box::new(
            ContactFilterHook,
        )));

        let physics_plugin =
            RapierPhysicsPlugin::<HookData>::default().with_default_system_setup(false);
        app.add_plugin(physics_plugin);

        app.add_network_stage_after(
            NetworkCoreStage::Update,
            PhysicsStages::SyncBackend,
            SystemStage::parallel().with_system_set(RapierPhysicsPlugin::<HookData>::get_systems(
                PhysicsStages::SyncBackend,
            )),
        );
        app.add_network_stage_after(
            PhysicsStages::SyncBackend,
            PhysicsStages::StepSimulation,
            SystemStage::parallel().with_system_set(RapierPhysicsPlugin::<HookData>::get_systems(
                PhysicsStages::StepSimulation,
            )),
        );
        app.add_network_stage_after(
            PhysicsStages::StepSimulation,
            PhysicsStages::Writeback,
            SystemStage::parallel().with_system_set(RapierPhysicsPlugin::<HookData>::get_systems(
                PhysicsStages::Writeback,
            )),
        );

        // NOTE: we run sync_removals at the end of the frame, too, in order to make sure we don’t miss any `RemovedComponents`.
        app.add_network_stage_before(
            NetworkCoreStage::Last,
            PhysicsStages::DetectDespawn,
            SystemStage::parallel().with_system_set(RapierPhysicsPlugin::<HookData>::get_systems(
                PhysicsStages::DetectDespawn,
            )),
        );

        app.add_network_system(cap_velocity);
        app.add_startup_system(modify_rapier_context);
    }
}
