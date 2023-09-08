use super::EffectVelocity;
use crate::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_rapier3d::parry::{math::Isometry, query::PointQuery, shape::TypedShape};

use std::{cmp::Ordering, f32::consts::PI};

#[derive(Component)]
pub struct VineEffect;

#[derive(Component)]
pub struct Vine;

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
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Vine potion effect
///
/// Goals:
/// - Grabs and joints dynamic bodies to other dynamic bodies
///   or to kinematic/fixed bodies.
/// - Travel upwards, away from gravity, if the slope is steep
///   enough.
/// - Burnable
pub fn vine_effect(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,

    ctx: Res<RapierContext>,
    vine_effect: Query<(), Or<(With<VineEffect>, With<Vine>)>>,
    potions: Query<(Entity, &GlobalTransform, Option<&EffectVelocity>), With<VineEffect>>,
    globals: Query<&GlobalTransform>,
    colliders: Query<&Collider>,
    mut gizmos: ResMut<RetainedGizmos>,
) {
    const DEBUG_TIME: f32 = 1000.0;

    let material = materials.add(StandardMaterial {
        base_color: Color::DARK_GREEN,
        perceptual_roughness: 0.2,
        ..default()
    });

    for (effect_entity, global, velocity) in &potions {
        commands.entity(effect_entity).remove::<VineEffect>();
    }

    let dt = ctx.integration_parameters.dt;
    for (effect_entity, global, velocity) in &potions {
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
        let vine_range = 2.0;
        let vine_radius = 0.05;
        let vine_height = 0.75;
        let half_height = vine_height / 2.0;

        gizmos.sphere(
            DEBUG_TIME,
            global.translation(),
            Quat::IDENTITY,
            vine_range,
            Color::PURPLE,
        );

        //let points = scatter_sampling(&*ctx, global.translation(), 500, vine_range, &mut *gizmos);
        let points = sunflower_sampling(&*ctx, global.translation(), 500, vine_range, &mut *gizmos);
        info!("points: {:?}", points.len());
        let groups = crate::objects::group_points(points, |center, ray, entities, rays| {
            let alignment = center.normal.dot(ray.normal);
            let center_y = center.normal;
            let (center_x, center_z) = center_y.any_orthonormal_pair();

            // cuboid
            /*
            let center_x = -center_x;
            let center_z = center_z;

            let offset = ray.point - center.point;
            let y = offset.dot(center_y);
            let x = offset.dot(center_x);
            let z = offset.dot(center_z);
            alignment >= 0.8
                && y.abs() <= vine_radius
                && x.abs() <= vine_radius
                && z.abs() <= vine_radius
                */

            // vine
            if let Some(initial_ray) = rays.get(1) {
                let vine_dir = (center.point - initial_ray.point).normalize_or_zero();
                let offset = center.point - ray.point;
                //info!("dir: {:?}", vine_dir);
                let vine_dot = vine_dir.dot(offset);
                let vine_point = vine_dir * vine_dot;
                let height_bounds = vine_dot.abs() < half_height;
                let radius_bounds = vine_point.distance(offset) < vine_radius;
                info!("radius: {:.1?}", vine_point);
                info!("radius: {:.1?}", ray.point);
                height_bounds && radius_bounds
            } else {
                center.point.distance(ray.point) <= vine_radius.max(half_height)
            }
        });
        let colors = crate::objects::debug_colors(groups.len());
        info!("groups: {:?}", groups.len());

        for (index, group) in groups.iter().enumerate() {
            let center_y = group.center.normal;
            let (center_x, center_z) = center_y.any_orthonormal_pair();
            let center_x = -center_x;
            let center_z = center_z;

            let color = colors[index % colors.len()];
            for ray in &group.points {
                gizmos.sphere(DEBUG_TIME, ray.point, Quat::IDENTITY, 0.05, color);
            }

            let vine_align = center_x.normalize_or_zero();

            //gizmos.ray(DEBUG_TIME, group.center.point, center_y * 0.5, Color::BLUE);
            //gizmos.ray(DEBUG_TIME, group.center.point, vine_align * 0.5, Color::RED);

            let default_size = 0.1;
            let mut min = Vec3::splat(-default_size / 2.0);
            let mut max = Vec3::splat(default_size / 2.0);

            let Some(initial_ray) = group.points.get(1) else { continue; };
            let dir = (group.center.point - initial_ray.point).normalize();
            for ray in &group.points {
                let offset = ray.point - group.center.point;
                let aligned = Vec3::new(
                    center_z.dot(offset),
                    center_y.dot(offset),
                    center_x.dot(offset),
                );
                min = min.min(aligned);
                max = max.max(aligned);
            }

            //gizmos.ray(DEBUG_TIME, group.center.point, center_y, Color::BLUE);
            //gizmos.ray(DEBUG_TIME, group.center.point, center_x, Color::RED);
            //gizmos.ray(DEBUG_TIME, group.center.point, center_z, Color::GREEN);

            // local space center

            let aabb = Aabb::from_min_max(min, max);
            let mut transform = Transform {
                translation: group.center.point, // + Vec3::from(aabb.center),
                //scale: Vec3::from(aabb.half_extents * 2.0),
                ..default()
            }
            .looking_to(center_y, dir);

            //let offset = transform.rotation * aabb.center;
            //transform.translation -= Vec3::from(offset);
            //gizmos.cuboid(DEBUG_TIME, transform, Color::CYAN);
            //gizmos.sphere(DEBUG_TIME, center.point - Vec3::from(aabb.center), Quat::IDENTITY, 0.05, Color::CYAN);
            //gizmos.sphere(DEBUG_TIME, center.point, Quat::IDENTITY, 0.05, Color::RED);
            gizmos.ray(DEBUG_TIME, group.center.point - dir * half_height, dir * half_height * 2.0,  color);
            commands
                .spawn(SpatialBundle {
                    transform: transform,
                    ..default()
                })
                .insert(Vine)
                //.insert(VineEffect)
                .insert(material.clone())
                .insert(RigidBody::Fixed)
                .insert(Sensor)
                .insert(ColliderBundle::collider(Collider::cylinder(half_height, vine_radius)));
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Scatter {
    pub from: Vec3,
    pub dir: Quat,

    pub angle: f32,
}

pub fn scatter_sampling(
    ctx: &RapierContext,
    from: Vec3,
    samples: usize,
    radius: f32,
    gizmos: &mut RetainedGizmos,
) -> Vec<(Entity, RayIntersection)> {
    let mut results = Vec::new();

    let scatter_dirs = |scatter: &Scatter| {
        let outwards = Quat::from_axis_angle(Vec3::Y, scatter.angle)
            * Quat::from_axis_angle(Vec3::X, scatter.angle);
        [
            scatter.dir,
            scatter.dir
                * Quat::from_axis_angle(Vec3::Y, scatter.angle)
                * Quat::from_axis_angle(Vec3::X, scatter.angle),
            scatter.dir
                * Quat::from_axis_angle(Vec3::Y, -scatter.angle)
                * Quat::from_axis_angle(Vec3::X, scatter.angle),
            scatter.dir
                * Quat::from_axis_angle(Vec3::Y, -scatter.angle)
                * Quat::from_axis_angle(Vec3::X, -scatter.angle),
            scatter.dir
                * Quat::from_axis_angle(Vec3::Y, scatter.angle)
                * Quat::from_axis_angle(Vec3::X, -scatter.angle),
            scatter.dir * Quat::from_axis_angle(Vec3::Y, scatter.angle),
            scatter.dir * Quat::from_axis_angle(Vec3::Y, -scatter.angle),
            scatter.dir * Quat::from_axis_angle(Vec3::X, scatter.angle),
            scatter.dir * Quat::from_axis_angle(Vec3::X, -scatter.angle),
            //scatter.dir * Quat::from_axis_angle(Vec3::X, scatter.angle * 1.5),
            //scatter.dir * Quat::from_axis_angle(Vec3::Y, 90f32.to_radians()) * outwards,
            //scatter.dir * Quat::from_axis_angle(Vec3::X, scatter.angle * 2.5),
            //scatter.dir * Quat::from_axis_angle(Vec3::Y, 180f32.to_radians()) * outwards,
            //scatter.dir * Quat::from_axis_angle(Vec3::X, scatter.angle * 3.5),
            //scatter.dir * Quat::from_axis_angle(Vec3::Y, 270f32.to_radians()) * outwards,
        ]
    };

    let mut scatters = Vec::new();

    let dirs = [
        Quat::from_axis_angle(Vec3::Y, 0f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, 90f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, 180f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, 270f32.to_radians()),
        Quat::from_axis_angle(Vec3::X, 90f32.to_radians()),
        Quat::from_axis_angle(Vec3::X, 270f32.to_radians()),
        //Quat::from_axis_angle(Vec3::Y, 180f32.to_radians()),
        //Quat::from_axis_angle(Vec3::Y, 270f32.to_radians()),
        //Quat::from_axis_angle(Vec3::Z, 0f32.to_radians()),
        /*
        Quat::from_axis_angle(Vec3::Z, 90f32.to_radians()),
        Quat::from_axis_angle(Vec3::Z, 180f32.to_radians()),
        Quat::from_axis_angle(Vec3::Z, 270f32.to_radians()),
        */

        /*
        Quat::from_axis_angle(Vec3::Y, 45f32.to_radians()) * Quat::from_axis_angle(Vec3::X, 45f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, 90f32.to_radians()) * Quat::from_axis_angle(Vec3::Y, 45f32.to_radians()) * Quat::from_axis_angle(Vec3::X, 45f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, 180f32.to_radians()) * Quat::from_axis_angle(Vec3::Y, 45f32.to_radians()) * Quat::from_axis_angle(Vec3::X, 45f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, 270f32.to_radians()) * Quat::from_axis_angle(Vec3::Y, 45f32.to_radians()) * Quat::from_axis_angle(Vec3::X, 45f32.to_radians()),
        */
        /*Quat::from_axis_angle(Vec3::Y, -45f32.to_radians()) * Quat::from_axis_angle(Vec3::X, 45f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, 45f32.to_radians()) * Quat::from_axis_angle(Vec3::X, -45f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, -45f32.to_radians()) * Quat::from_axis_angle(Vec3::X, -45f32.to_radians()),

        Quat::from_axis_angle(Vec3::Y, 135f32.to_radians()) * Quat::from_axis_angle(Vec3::X, 45f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, -135f32.to_radians()) * Quat::from_axis_angle(Vec3::X, 45f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, 135f32.to_radians()) * Quat::from_axis_angle(Vec3::X, -45f32.to_radians()),
        Quat::from_axis_angle(Vec3::Y, -135f32.to_radians()) * Quat::from_axis_angle(Vec3::X, -45f32.to_radians()),
        */
    ];
    scatters.extend(
        dirs.iter()
            .map(|dir| Scatter {
                from,
                dir: *dir,
                angle: 45f32.to_radians(),
            })
            .map(|scatter| {
                scatter_dirs(&scatter)
                    .iter()
                    .map(|dir| Scatter {
                        from: scatter.from,
                        dir: *dir,
                        angle: scatter.angle / 2.0,
                    })
                    .collect::<Vec<_>>()
            })
            .flatten(),
    );

    let step = radius / 4.0;
    while let Some(scatter) = scatters.pop() {
        if scatter.angle < 1f32.to_radians() || scatter.from.distance(from) + 0.25 >= radius {
            continue;
        }

        //let color = colors[(step as usize % colors.len()) - 1];

        let mut toi = step;
        if let Some((entity, ray)) = ctx.cast_ray_and_get_normal(
            scatter.from,
            scatter.dir * Vec3::NEG_Z,
            step,
            true,
            QueryFilter::default().exclude_sensors(),
        ) {
            toi = ray.toi;
            results.push((entity, ray));
        } else {
            let traveled = scatter.from + scatter.dir * Vec3::NEG_Z * step;

            let new = scatter_dirs(&scatter);

            for new_dir in new {
                scatters.push(Scatter {
                    from: traveled,
                    dir: new_dir.normalize(),
                    angle: scatter.angle / 2.0,
                });
            }
        }

        /*
        gizmos.ray(
            1000.0,
            scatter.from,
            scatter.dir * Vec3::NEG_Z * toi,
            Color::CRIMSON,
        );
        */
    }

    results
}

fn biased_orthonormal_basis(up: Vec3) -> (Vec3, Vec3) {
    let (x, z) = up.any_orthonormal_pair();
    (bias_vec(x), bias_vec(z))
}

fn bias_vec(vec: Vec3) -> Vec3 {
    bias_vec_basis(vec, Vec3::Y, Vec3::X, Vec3::Z)
}

fn bias_vec_basis(vec: Vec3, up: Vec3, right: Vec3, back: Vec3) -> Vec3 {
    let mut biased = [
        vec.project_onto(up),
        vec.project_onto(right),
        vec.project_onto(back),
    ];

    // sort by longest
    biased.sort_by(|a, b| {
        b.length()
            .partial_cmp(&a.length())
            .unwrap_or(Ordering::Less)
    });
    (biased[0] + biased[1]).normalize_or_zero()
}

pub fn sunflower_sampling(
    ctx: &RapierContext,
    from: Vec3,
    samples: usize,
    radius: f32,
    gizmos: &mut RetainedGizmos,
) -> Vec<(Entity, RayIntersection)> {
    let mut results = Vec::new();

    for point in crate::objects::sunflower_circle(samples, 0.0) {
        let point = Vec3::new(point.x, 0.0, point.y) * radius;

        if let Some((entity, ray)) = ctx.cast_ray_and_get_normal(
            from + point,
            -Vec3::Y,
            radius,
            true,
            QueryFilter::default().exclude_sensors(),
        ) {
            results.push((entity, ray));
        }
    }

    results
}
