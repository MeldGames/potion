/// Collision Grouping Flags
use bevy::{
    ecs::{query::WorldQuery, system::EntityCommands},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

pub mod contact_filter;
pub mod context_ext;
pub mod joint_break;
pub mod joint_interpolation;
pub mod muscle;
pub mod slot;
pub mod split_compound;

pub mod prelude {
    pub use super::{
        contact_filter::*, context_ext::*, joint_break::*, joint_interpolation::*, muscle::*,
        slot::*, ColliderBundle, RigidBodyBundle, GRAB_GROUPING, PLAYER_GROUPING, REST_GROUPING,
        STORED_GROUPING, TERRAIN_GROUPING,
    };
}

use prelude::*;

use crate::physics::joint_interpolation::JointInterpolationPlugin;

bitflags::bitflags! {
    pub struct Groups: u32 {
        const PLAYER = 1 << 0;
        const TERRAIN = 1 << 1;
        const FLUFF = 1 << 3;
        const STORED = 1 << 5;

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

pub const STORED_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::STORED.bits()),
    Group::from_bits_truncate(0),
);

#[derive(Bundle)]
pub struct RigidBodyBundle {
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub additional_mass_properties: AdditionalMassProperties,
    pub read_mass_properties: ReadMassProperties,
    pub sleeping: Sleeping,
    pub damping: Damping,
    pub ccd: Ccd,
    pub friction: Friction,
    pub restitution: Restitution,
}

impl RigidBodyBundle {
    pub fn dynamic() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            ..default()
        }
    }

    pub fn fixed() -> Self {
        Self {
            rigid_body: RigidBody::Fixed,
            ..default()
        }
    }

    pub fn kinematic_position() -> Self {
        Self {
            rigid_body: RigidBody::KinematicPositionBased,
            ..default()
        }
    }

    pub fn kinematic_velocity() -> Self {
        Self {
            rigid_body: RigidBody::KinematicVelocityBased,
            ..default()
        }
    }
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
            sleeping: Sleeping::default(),
            damping: Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            ccd: Ccd::default(),
            friction: Friction::default(),
            restitution: Restitution::coefficient(0.45),
        }
    }
}

#[derive(Bundle)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub mass_properties: ColliderMassProperties,
    //pub colliding_entities: CollidingEntities,
    pub collision_groups: CollisionGroups,
    pub solver_groups: SolverGroups,
}

impl ColliderBundle {
    pub fn collider(collider: impl Into<Collider>) -> Self {
        Self {
            collider: collider.into(),
            ..default()
        }
    }
}

impl Default for ColliderBundle {
    fn default() -> Self {
        Self {
            collider: Collider::default(),
            mass_properties: ColliderMassProperties::default(),
            collision_groups: CollisionGroups::default(),
            solver_groups: SolverGroups::default(),
        }
    }
}

#[derive(WorldQuery)]
pub struct FillRigidBodyComponents {
    pub rigid_body: &'static RigidBody,
    pub velocity: Option<&'static Velocity>,
    pub additional_mass_properties: Option<&'static AdditionalMassProperties>,
    pub read_mass_properties: Option<&'static ReadMassProperties>,
    pub external_force: Option<&'static ExternalForce>,
    pub external_impulse: Option<&'static ExternalImpulse>,
    pub sleeping: Option<&'static Sleeping>,
    pub damping: Option<&'static Damping>,
    pub ccd: Option<&'static Ccd>,
    pub colliding_entities: Option<&'static CollidingEntities>,
    pub friction: Option<&'static Friction>,
    pub restitution: Option<&'static Restitution>,
    pub collision_groups: Option<&'static CollisionGroups>,
    pub solver_groups: Option<&'static SolverGroups>,
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
        if let None = self.ccd {
            commands.insert(Ccd::default());
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
    }
}

pub fn fill_missing(
    mut commands: Commands,
    to_fill: Query<(Entity, FillRigidBodyComponents), Added<RigidBody>>,
) {
    for (entity, components) in &to_fill {
        components.fill_missing(&mut commands.entity(entity));
    }
}

pub fn modify_rapier_context(mut context: ResMut<RapierContext>) {
    let integration = &mut context.integration_parameters;
    /*
    integration.damping_ratio = 0.5;
    integration.joint_erp = 0.8;
    integration.joint_damping_ratio = 0.5;
    */
    // Try to avoid launching players in weird situations
    integration.max_penetration_correction = 1000.0;
    integration.dt = crate::TICK_RATE.as_secs_f32();
}

pub const VELOCITY_CAP: f32 = 50.0;
pub const ANG_VELOCITY_CAP: f32 = 5.0;

pub fn cap_velocity(mut velocities: Query<&mut Velocity, Changed<Velocity>>) {
    for mut velocity in &mut velocities {
        velocity.linvel = velocity.linvel.clamp_length_max(VELOCITY_CAP);
        velocity.angvel = velocity.angvel.clamp_length_max(ANG_VELOCITY_CAP);
    }
}

pub const IMPULSE_CAP: f32 = 0.001;
pub const ANG_IMPULSE_CAP: f32 = 0.0001;

pub fn cap_impulse(
    mut impulses: Query<(&mut ExternalImpulse, &ReadMassProperties), Changed<ExternalImpulse>>,
) {
    for (mut impulse, mass) in &mut impulses {
        if mass.0.mass < 2.0 {
            impulse.impulse = impulse
                .impulse
                .clamp_length_max(IMPULSE_CAP / 500.0 / mass.0.mass);
            impulse.torque_impulse = impulse
                .torque_impulse
                .clamp_length_max(ANG_IMPULSE_CAP / 100.0 / mass.0.mass);
        } else {
            impulse.impulse = impulse.impulse.clamp_length_max(IMPULSE_CAP);
            impulse.torque_impulse = impulse.torque_impulse.clamp_length_max(ANG_IMPULSE_CAP);
        }
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
        if translation.length() > 50_000.0f32 {
            warn!("Entity {:?} went too far out", name);
            commands.entity(entity).remove::<RigidBody>();
        }
    }
}

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReadMassProperties>()
            .register_type::<ColliderMassProperties>();

        app.insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::Fixed {
                dt: crate::TICK_RATE.as_secs_f32() / 1.0,
                substeps: 8,
            },
            ..Default::default()
        });

        type PhysicsPlugin<'w, 's> = RapierPhysicsPlugin<ContactFilterHook<'w, 's>>;
        let physics_plugin = PhysicsPlugin::default().with_default_system_setup(false);
        app.add_plugins(physics_plugin);

        app.configure_sets(
            FixedUpdate,
            (
                PhysicsSet::SyncBackend,
                PhysicsSet::SyncBackendFlush,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback,
            )
                .chain(),
        );

        app.add_systems(PostUpdate, bevy_rapier3d::plugin::systems::sync_removals);

        app.add_systems(
            FixedUpdate,
            PhysicsPlugin::get_systems(PhysicsSet::SyncBackend).in_set(PhysicsSet::SyncBackend),
        );
        app.add_systems(
            FixedUpdate,
            PhysicsPlugin::get_systems(PhysicsSet::SyncBackendFlush)
                .in_set(PhysicsSet::SyncBackendFlush),
        );
        app.add_systems(
            FixedUpdate,
            PhysicsPlugin::get_systems(PhysicsSet::StepSimulation)
                .in_set(PhysicsSet::StepSimulation),
        );
        app.add_systems(
            FixedUpdate,
            PhysicsPlugin::get_systems(PhysicsSet::Writeback).in_set(PhysicsSet::Writeback),
        );

        /*
        app.add_systems(Update, fill_missing);
        app.add_systems(Update, cap_velocity)
            .add_systems(Update, cap_impulse);
        */
        app.add_systems(Update, prevent_oob);
        app.add_systems(Update, minimum_mass);
        app.add_systems(Startup, modify_rapier_context);
        //app.add_systems(Update, split_compound::split_compound);

        app.add_plugins(MusclePlugin);
        app.add_plugins(BreakJointPlugin);
        app.add_plugins(SlotPlugin);
        app.add_plugins(JointInterpolationPlugin);
    }
}

const MINIMUM_MASS: f32 = 0.1;
pub fn minimum_mass(
    mut masses: Query<
        (
            DebugName,
            &RigidBody,
            &mut ColliderMassProperties,
            &mut Damping,
            &ReadMassProperties,
        ),
        Changed<ReadMassProperties>,
    >,
) {
    for (name, body, mut mass, mut damping, read) in &mut masses {
        if *body != RigidBody::Dynamic {
            continue;
        }

        if read.0.mass == 0.0 {
            continue;
        }

        let mut scale = 1.0;
        if read.0.mass < MINIMUM_MASS {
            scale = MINIMUM_MASS / read.0.mass;
        }

        if scale != 1.0 || read.0.mass <= 0.0 {
            let mut new_mass = read.0;
            if read.0.mass == 0.0 {
                new_mass.mass = MINIMUM_MASS;
                new_mass.principal_inertia = Vec3::splat(MINIMUM_MASS);
            } else {
                new_mass.mass *= scale;
                new_mass.principal_inertia *= scale * scale;
            }

            info!(
                "changed mass for {:?}: {:.2?} -> {:.2?}",
                name, read.0.mass, new_mass.mass
            );
            *mass = ColliderMassProperties::MassProperties(new_mass);
            damping.angular_damping += 1.0;
        }
    }
}

use bevy_rapier3d::parry::{query::{PersistentQueryDispatcher, DefaultQueryDispatcher}, bounding_volume::BoundingVolume};
use bevy_rapier3d::rapier::geometry::ContactManifold;
use bevy_rapier3d::na::Isometry3;

/// Get a list of contacts for a given shape.
pub fn contact_manifolds(
    ctx: &RapierContext,
    position: Vec3,
    rotation: Quat,
    collider: &Collider,
    filter: &QueryFilter,
) -> Vec<(Entity, ContactManifold)> {
    const FUDGE: f32 = 0.05;

    let physics_scale = ctx.physics_scale();

    let shape = &collider.raw;
    let shape_iso = Isometry3 {
        translation: (position * physics_scale).into(),
        rotation: rotation.into(),
    };

    let shape_aabb = shape.compute_aabb(&shape_iso).loosened(FUDGE);

    let mut manifolds = Vec::new();
    ctx.query_pipeline
        .colliders_with_aabb_intersecting_aabb(&shape_aabb, |handle| {
            if let Some(collider) = ctx.colliders.get(*handle) {
                if RapierContext::with_query_filter(&ctx, *filter, |rapier_filter| {
                    rapier_filter.test(&ctx.bodies, *handle, collider)
                }) {
                    let mut new_manifolds = Vec::new();
                    let pos12 = shape_iso.inv_mul(collider.position());
                    let _ = DefaultQueryDispatcher.contact_manifolds(
                        &pos12,
                        shape.as_ref(),
                        collider.shape(),
                        0.01,
                        &mut new_manifolds,
                        &mut None,
                    );

                    if let Some(entity) = ctx.collider_entity(*handle) {
                        manifolds
                            .extend(new_manifolds.into_iter().map(|manifold| (entity, manifold)));
                    }
                }
            }

            true
        });

    manifolds
}
