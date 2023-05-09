/// Collision Grouping Flags
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod contact_filter;
pub mod muscle;
pub mod spring;

pub use contact_filter::*;
pub use muscle::*;
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

#[derive(Bundle)]
pub struct RigidBodyBundle {
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub additional_mass_properties: AdditionalMassProperties,
    pub read_mass_properties: ReadMassProperties,
    pub locked_axes: LockedAxes,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub sleeping: Sleeping,
    pub damping: Damping,
    pub dominance: Dominance,
    pub ccd: Ccd,
    pub gravity_scale: GravityScale,
    pub colliding_entities: CollidingEntities,
    pub sensor: Sensor,
    pub friction: Friction,
    pub restitution: Restitution,
    pub collision_groups: CollisionGroups,
    pub solver_groups: SolverGroups,
    pub contact_force_event_threshold: ContactForceEventThreshold,
}

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

        type PhysicsPlugin<'w, 's> = RapierPhysicsPlugin<ContactFilterHook<'w, 's>>;
        let physics_plugin = PhysicsPlugin::default()
            .with_default_system_setup(false);
        app.add_plugin(physics_plugin);

        app.world
            .resource_mut::<Schedules>()
            .get_mut(&CoreSchedule::FixedUpdate)
            .unwrap()
            .configure_sets(
                (
                    PhysicsSet::SyncBackend,
                    PhysicsSet::SyncBackendFlush,
                    PhysicsSet::StepSimulation,
                    PhysicsSet::Writeback,
                )
                    .chain(),
            );

        app.add_system(
            bevy_rapier3d::plugin::systems::sync_removals
                .in_base_set(CoreSet::PostUpdate)
        );

        app.add_systems(
            PhysicsPlugin::get_systems(PhysicsSet::SyncBackend)
                .in_base_set(PhysicsSet::SyncBackend)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
        app.add_systems(
            PhysicsPlugin::get_systems(PhysicsSet::SyncBackendFlush)
                .in_base_set(PhysicsSet::SyncBackendFlush)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
        app.add_systems(
            PhysicsPlugin::get_systems(PhysicsSet::StepSimulation)
                .in_base_set(PhysicsSet::StepSimulation)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
        app.add_systems(
            PhysicsPlugin::get_systems(PhysicsSet::Writeback)
                .in_base_set(PhysicsSet::Writeback)
                .in_schedule(CoreSchedule::FixedUpdate),
        );

        app.add_system(cap_velocity);
        app.add_startup_system(modify_rapier_context);
    }
}
