/// Vine potion effect
///
/// Goals:
/// - Grabs and joints dynamic bodies to other dynamic bodies
///   or to kinematic/fixed bodies.
/// - Travel upwards, away from gravity, if the slope is steep
///   enough.
/// - Burnable
use super::{spiral_sphere, EffectVelocity};
use crate::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_rapier3d::parry::{
    math::Isometry,
    query::{NonlinearRigidMotion, PointQuery},
    shape::TypedShape,
};

use std::{cmp::Ordering, f32::consts::PI};

#[derive(Component, Clone)]
pub struct VineEffect {
    pub vine: Vine,
    pub explode_radius: f32,
}

impl Default for VineEffect {
    fn default() -> Self {
        Self {
            vine: Vine::default(),
            explode_radius: 2.0,
        }
    }
}

#[derive(Clone)]
pub struct VineGrowth {
    pub point: Vec3,
    pub direction: Vec3,
}

#[derive(Component, Clone)]
pub struct Vine {
    /// How many links this vine can generate.
    pub growth: usize,

    /// Local-space points/direction that can be used for generating new links in
    /// the vine chain.
    pub growth_points: Vec<VineGrowth>,

    /// Radius of the cylinder that makes up this vine.
    pub radius: f32,
    /// Height of the cylinder that makes up this vine.
    pub height: f32,

    /// Generated from this vine.
    pub parent: Option<Entity>,
    /// Root vine this vine comes from.
    pub root: Option<Entity>,
}

impl Vine {
    pub fn half_height(&self) -> f32 {
        self.height / 2.0
    }

    pub fn basic_growth_points(&self) -> Vec<VineGrowth> {
        vec![
            VineGrowth {
                point: Vec3::new(0.0, self.half_height(), 0.0),
                direction: Vec3::Y,
            },
            /*
            VineGrowth {
                point: Vec3::new(0.0, -self.half_height(), 0.0),
                direction: -Vec3::Y,
            },
            */
        ]
    }

    pub fn collider(&self) -> Collider {
        Collider::cylinder(self.half_height(), self.radius)
    }

    pub fn grow(&mut self) -> Self {
        self.growth = self.growth.saturating_sub(1);
        let mut growth = self.clone();
        growth
    }
}

impl Default for Vine {
    fn default() -> Self {
        Self {
            growth: 20,
            growth_points: Vec::new(),
            radius: 0.05,
            height: 0.65,
            parent: None,
            root: None,
        }
    }
}

pub fn sunflower_effect(mut gizmos: Gizmos) {
    for point in super::sunflower_circle(500, 0.0) {
        let shifted = Vec3::Y;
        let point = shifted + Vec3::new(point.x, 0.0, point.y);
        gizmos.sphere(point, Quat::IDENTITY, 0.01, Color::ORANGE);
    }

    for point in super::spiral_sphere(500) {
        let shifted = Vec3::Y * 2.0 + Vec3::Z * 2.0;
        let point = shifted + point;
        gizmos.sphere(point, Quat::IDENTITY, 0.01, Color::ORANGE);
    }
}

/// Despawn a vine if it isn't in contact with anything
/// other than another vine.
pub fn vine_despawn(
    mut commands: Commands,
    ctx: Res<RapierContext>,
    vines: Query<(Entity, &GlobalTransform, &Collider), With<Vine>>,
) {
    for (entity, global, collider) in &vines {
        let manifolds = crate::physics::contact_manifolds(
            &*ctx,
            global.translation(),
            Quat::IDENTITY,
            collider,
            &QueryFilter::default().exclude_sensors(),
        );

        let mut despawn = true;
        for (contact_entity, _) in manifolds {
            if vines.contains(contact_entity) {
                continue;
            }

            despawn = false;
        }

        if despawn {
            //commands.entity(entity).despawn_recursive();
        }
    }
}

const DEBUG_TIME: f32 = 1000.0;

/// Spawn the initial vines given a radius, velocity, etc.
pub fn vine_effect(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,

    ctx: Res<RapierContext>,
    mut potions: Query<(
        Entity,
        &VineEffect,
        &GlobalTransform,
        Option<&EffectVelocity>,
    )>,
    globals: Query<&GlobalTransform>,
    colliders: Query<&Collider>,
    mut gizmos: ResMut<RetainedGizmos>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::DARK_GREEN,
        perceptual_roughness: 0.2,
        ..default()
    });

    let colors = crate::objects::debug_colors(5);

    let dt = ctx.integration_parameters.dt;
    for (effect_entity, mut vine_effect, global, velocity) in &mut potions {
        let mut vine = vine_effect.vine.clone();
        vine.growth_points = vine.basic_growth_points();

        commands.entity(effect_entity).remove::<VineEffect>();
        if vine.growth == 0 {
            continue;
        }

        let velocity = if let Some(velocity) = velocity {
            if velocity.linear.length_squared() == 0.0 {
                Vec3::NEG_Y
            } else {
                velocity.linear
            }
        } else {
            Vec3::NEG_Y
        };

        //let effect_radius = 3.0;

        /*
                gizmos.sphere(
                    DEBUG_TIME,
                    global.translation(),
                    Quat::IDENTITY,
                    vine_range,
                    Color::PURPLE,
                );
        */

        if let Some((entity, ray)) = ctx.cast_ray_and_get_normal(
            global.translation(),
            -Vec3::Y,
            vine_effect.explode_radius,
            true,
            QueryFilter::default().exclude_sensors(),
        ) {
            let normal = ray.normal;
            let (center_x, center_z) = normal.any_orthonormal_pair();

            let mut rotation = Transform::default().looking_to(normal, center_x).rotation;

            for step in 0..3 {
                let vine_offset = rotation * (Vec3::Y * vine.half_height());
                let growth_stem = rotation * (Vec3::Y * vine.height);

                let color = colors[vine_effect.vine.growth % colors.len()];
                commands
                    .spawn((SpatialBundle {
                        transform: Transform {
                            translation: ray.point + vine_offset + ray.normal * vine.radius,
                            rotation: rotation,
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("Vine"),
                    vine.clone(),
                    material.clone(),
                    RigidBody::Fixed,
                    Sensor,
                    ColliderDebugColor(color),
                    ColliderBundle::collider(vine.collider()),
                ));

                rotation = rotation * Quat::from_axis_angle(Vec3::Z, 45f32.to_radians());
            }
        }
    }
}

pub fn vine_growth(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,

    ctx: Res<RapierContext>,
    mut vines: Query<(Entity, &mut Vine, &GlobalTransform)>,
    globals: Query<&GlobalTransform>,
    colliders: Query<&Collider>,
    mut gizmos: ResMut<RetainedGizmos>,
) {
    for (entity, mut vine, global) in &mut vines {
        for growth in vine.growth_points.clone() {
            if vine.growth == 0 {
                break;
            }

            let vine = vine.grow();
            let transform = global.compute_transform();
            let global_direction = transform.rotation * growth.direction;
            let global_growth_point = global.transform_point(growth.point);
            let collider = vine.collider();

            gizmos.ray(
                DEBUG_TIME,
                global_growth_point,
                global_direction * vine.height,
                Color::PURPLE,
            );
            // Cast ray in direction to check for obstacles
            if let Some((entity, ray)) = ctx.cast_ray_and_get_normal(
                global_growth_point,
                global_direction,
                vine.height,
                true,
                QueryFilter::default().exclude_sensors(),
            ) {
                info!("ray: {:?}", ray);
                // Attach the vine to the hit obstacle
                commands.spawn((
                    SpatialBundle {
                        transform: Transform {
                            translation: global_growth_point + global_direction * ray.toi,
                            rotation: transform.rotation,
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("Vine"),
                    Sensor,
                    ColliderBundle::collider(collider),
                    //ColliderDebugColor(color),
                    //RigidBody::Fixed,
                    vine,
                ));
            } else {
                // Cast shape with angular velocity towards gravity
                let adjusted_global = Transform {
                    translation: global_growth_point + global_direction * vine.half_height(),
                    rotation: transform.rotation,
                    ..default()
                };
                let local_center = Vec3::new(0.0, -vine.half_height(), 0.0);
                let global_center = adjusted_global.transform_point(local_center);
                gizmos.sphere(DEBUG_TIME, global_center, Quat::IDENTITY, 0.02, Color::CYAN);

                let current = transform.rotation * Vec3::Y;
                let dangling = -Vec3::Y;
                let axis = current.cross(dangling);

                gizmos.ray(DEBUG_TIME, global_center, axis * 0.5, Color::GREEN);
                gizmos.ray(DEBUG_TIME, global_center, -axis * 0.5, Color::GREEN);

                let time = if let Some((collider, toi)) = ctx.query_pipeline.nonlinear_cast_shape(
                    &ctx.bodies,
                    &ctx.colliders,
                    &NonlinearRigidMotion {
                        start: Isometry {
                            translation: adjusted_global.translation.into(),
                            rotation: adjusted_global.rotation.into(),
                        },
                        local_center: local_center.into(),
                        linvel: Vec3::ZERO.into(),
                        angvel: axis.into(),
                    },
                    collider.raw.as_ref(),
                    0.0,
                    1.0,
                    true,
                    bevy_rapier3d::rapier::pipeline::QueryFilter::default().exclude_sensors(),
                ) {
                    toi.toi
                } else {
                    1.0
                };

                let local_translation = adjusted_global.translation - global_center;
                let rotation = Quat::from_axis_angle(axis, time);
                let rotated_translation = global_center + rotation * local_translation;
                let rotated_rotation = rotation * adjusted_global.rotation;
                //let rotated_translation = adjusted_global.translation + axis.cross()
                //gizmos.ray(DEBUG_TIME, rotated_translation, rotated_rotation * Vec3::Y * vine.half_height(), Color::PURPLE);
                commands.spawn((
                    SpatialBundle {
                        transform: Transform {
                            //translation: adjusted_global.translation,
                            translation: rotated_translation,
                            rotation: rotation * adjusted_global.rotation,
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("Vine"),
                    ColliderBundle::collider(collider),
                    //RigidBody::Fixed,
                    Sensor,
                    //ColliderDebugColor(color)),
                    vine,
                ));
            };

            // Cast around the end of the

            /*
            let minimum = global_growth_point - Vec3::splat(vine.height);
            let maximum = global_growth_point + Vec3::splat(vine.height);
            let aabb = Aabb::from_min_max(minimum, maximum);

            ctx.move_shape();
            */
            /*crate::physics::contact_manifolds(&*ctx, global_growth_point)
            ctx.colliders_with_aabb_intersecting_aabb(aabb, |entity| -> bool {
                let collider = colliders.get(entity).unwrap();
                true
            });*/
        }

        vine.growth_points = Vec::new();
    }
}