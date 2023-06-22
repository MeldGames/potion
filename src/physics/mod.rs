/// Collision Grouping Flags
use bevy::{
    ecs::{query::WorldQuery, system::EntityCommands},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

pub mod contact_filter;
pub mod joint_break;
pub mod muscle;
pub mod slot;

pub use contact_filter::*;
pub use muscle::*;

bitflags::bitflags! {
    pub struct Groups: u32 {
        const PLAYER = 1 << 0;
        const TERRAIN = 1 << 1;
        const FLUFF = 1 << 3;

        const PLAYER_FILTER = Groups::TERRAIN.bits();
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
    Group::from_bits_truncate(0),
    //Group::from_bits_truncate(Groups::PLAYER.bits()),
);
pub const GRAB_GROUPING: CollisionGroups = PLAYER_GROUPING;

#[derive(Bundle)]
pub struct RigidBodyBundle {
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub additional_mass_properties: AdditionalMassProperties,
    pub read_mass_properties: ReadMassProperties,
    pub locked_axes: LockedAxes,
    pub sleeping: Sleeping,
    pub damping: Damping,
    pub dominance: Dominance,
    pub ccd: Ccd,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub restitution: Restitution,
}

impl Default for RigidBodyBundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            velocity: Velocity::default(),
            external_force: ExternalForce::default(),
            external_impulse: ExternalImpulse::default(),
            additional_mass_properties: AdditionalMassProperties::default(),
            read_mass_properties: ReadMassProperties::default(),
            locked_axes: LockedAxes::default(),
            sleeping: Sleeping::default(),
            damping: Damping::default(),
            dominance: Dominance::default(),
            ccd: Ccd::default(),
            gravity_scale: GravityScale::default(),
            friction: Friction::default(),
            restitution: Restitution::default(),
        }
    }
}

#[derive(Bundle)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub collider_mass_properties: ColliderMassProperties,
    pub colliding_entities: CollidingEntities,
    pub collision_groups: CollisionGroups,
    pub solver_groups: SolverGroups,
    pub contact_force_event_threshold: ContactForceEventThreshold,
}

impl Default for ColliderBundle {
    fn default() -> Self {
        Self {
            collider: Collider::default(),
            collider_mass_properties: ColliderMassProperties::default(),
            colliding_entities: CollidingEntities::default(),
            collision_groups: CollisionGroups::default(),
            solver_groups: SolverGroups::default(),
            contact_force_event_threshold: ContactForceEventThreshold::default(),
        }
    }
}

#[derive(WorldQuery)]
pub struct FillRigidBodyComponents {
    pub rigid_body: &'static RigidBody,
    pub velocity: Option<&'static Velocity>,
    pub additional_mass_properties: Option<&'static AdditionalMassProperties>,
    pub read_mass_properties: Option<&'static ReadMassProperties>,
    pub locked_axes: Option<&'static LockedAxes>,
    pub external_force: Option<&'static ExternalForce>,
    pub external_impulse: Option<&'static ExternalImpulse>,
    pub sleeping: Option<&'static Sleeping>,
    pub damping: Option<&'static Damping>,
    pub dominance: Option<&'static Dominance>,
    pub ccd: Option<&'static Ccd>,
    pub gravity_scale: Option<&'static GravityScale>,
    pub colliding_entities: Option<&'static CollidingEntities>,
    pub friction: Option<&'static Friction>,
    pub restitution: Option<&'static Restitution>,
    pub collision_groups: Option<&'static CollisionGroups>,
    pub solver_groups: Option<&'static SolverGroups>,
    pub contact_force_event_threshold: Option<&'static ContactForceEventThreshold>,
}

impl<'a> FillRigidBodyComponentsItem<'a> {
    pub fn fill_missing(&self, commands: &mut EntityCommands) {
        if let None = self.velocity {
            commands.insert(Velocity::default());
        }
        if let None = self.additional_mass_properties {
            commands.insert(AdditionalMassProperties::default());
        }
        if let None = self.read_mass_properties {
            commands.insert(ReadMassProperties::default());
        }
        if let None = self.locked_axes {
            commands.insert(LockedAxes::default());
        }
        if let None = self.external_force {
            commands.insert(ExternalForce::default());
        }
        if let None = self.external_impulse {
            commands.insert(ExternalImpulse::default());
        }
        if let None = self.sleeping {
            commands.insert(Sleeping::default());
        }
        if let None = self.damping {
            commands.insert(Damping::default());
        }
        if let None = self.dominance {
            commands.insert(Dominance::default());
        }
        if let None = self.ccd {
            commands.insert(Ccd::default());
        }
        if let None = self.gravity_scale {
            commands.insert(GravityScale::default());
        }
        if let None = self.colliding_entities {
            commands.insert(CollidingEntities::default());
        }
        if let None = self.friction {
            commands.insert(Friction::default());
        }
        if let None = self.restitution {
            commands.insert(Restitution::default());
        }
        if let None = self.collision_groups {
            commands.insert(CollisionGroups::default());
        }
        if let None = self.solver_groups {
            commands.insert(SolverGroups::default());
        }
        if let None = self.contact_force_event_threshold {
            commands.insert(ContactForceEventThreshold::default());
        }
    }
}

pub fn fill_missing(
    mut commands: Commands,
    to_fill: Query<(Entity, FillRigidBodyComponents), Added<RigidBody>>,
) {
    for (entity, components) in &to_fill {
        info!("filling missing");
        components.fill_missing(&mut commands.entity(entity));
    }
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

pub fn prevent_oob(
    mut commands: Commands,
    bodies: Query<
        (Entity, DebugName, &GlobalTransform),
        (Changed<GlobalTransform>, With<RigidBody>),
    >,
) {
    for (entity, name, position) in &bodies {
        let translation = position.translation();
        if translation.length() > 100_000.0f32 {
            warn!("Entity {:?} went too far out", name);
            commands.entity(entity).remove::<RigidBody>();
        }
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
        let physics_plugin = PhysicsPlugin::default().with_default_system_setup(false);
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
            bevy_rapier3d::plugin::systems::sync_removals.in_base_set(CoreSet::PostUpdate),
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

        app.add_system(fill_missing);
        app.add_system(cap_velocity);
        app.add_system(prevent_oob);
        app.add_startup_system(modify_rapier_context);
    }
}
