use std::cmp::Ordering;

use crate::prelude::*;

pub mod vine;

#[derive(Component)]
pub struct EffectVelocity {
    pub linear: Vec3,
}

pub struct EffectPlugin;
impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, crate::previous::previous::<Velocity>);

        app.add_systems(
            FixedUpdate,
            (vine::vine_effect, vine::vine_growth, vine::vine_despawn),
        );
        app.add_systems(Update, vine::sunflower_effect);
    }
}

// helper methods

/// Uniform "sunflower seeding" sampling in a circle.
pub fn sunflower_circle(samples: usize, boundary_smoothing: f32) -> Vec<Vec2> {
    let n = samples;
    let alpha = boundary_smoothing;
    let phi: f32 = (1.0 + 5.0f32.sqrt()) / 2.0;
    let mut points = Vec::new();

    let angle_stride = 360.0 * phi;
    let boundary_points = alpha * (n as f32).sqrt();
    for k in 1..(n + 1) {
        let r = boundary_radius(k as f32, n as f32, boundary_points);
        let theta = k as f32 * angle_stride;
        points.push(Vec2::new(r * theta.cos(), r * theta.sin()));
    }

    points
}

/// Uniform sampling in a sphere.
pub fn spiral_sphere(samples: usize) -> Vec<Vec3> {
    let n = samples;
    let phi: f32 = (1.0 + 5.0f32.sqrt()) / 2.0;

    let mut points = Vec::new();
    for i in 0..n {
        let i = i as f32;
        let y = 1.0 - (i / (n - 1) as f32) * 2.0;
        let radius = (1.0 - y * y).sqrt();
        let theta = phi * i;

        let x = theta.cos() * radius;
        let z = theta.sin() * radius;
        points.push(Vec3::new(x, y, z));
    }

    points
}

pub fn boundary_radius(k: f32, n: f32, b: f32) -> f32 {
    if k > n - b {
        // put on the boundary
        1.0
    } else {
        // apply square root
        (k - 1.0 / 2.0).sqrt() / (n - (b + 1.0) / 2.0).sqrt()
    }
}

pub fn shape_closest_point(
    collider_global: &GlobalTransform,
    collider: &Collider,
    point: Vec3,
) -> Vec3 {
    use bevy_rapier3d::parry::{math::Isometry, query::PointQuery, shape::TypedShape};

    let (_, rotation, translation) = collider_global.to_scale_rotation_translation();
    let iso = Isometry {
        translation: translation.into(),
        rotation: rotation.into(),
    };

    let point_projection = match collider.raw.as_typed_shape() {
        TypedShape::Ball(ball) => ball.project_point(&iso, &point.into(), true),
        TypedShape::Cuboid(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::Cylinder(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::ConvexPolyhedron(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::TriMesh(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::Capsule(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::Compound(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::HalfSpace(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::Cone(raw) => raw.project_point(&iso, &point.into(), true),
        TypedShape::HeightField(raw) => raw.project_point(&iso, &point.into(), true),
        _ => {
            unimplemented!("{:?}", collider.raw.shape_type());
        }
    };

    point_projection.point.into()
}

#[derive(Debug, Clone)]
pub struct Group {
    pub center_entity: Entity,
    pub center: RayIntersection,
    pub entities: Vec<Entity>,
    pub points: Vec<RayIntersection>,
}

impl Group {
    pub fn average_normal(&self) -> Vec3 {
        let sum = self.points.iter().map(|ray| ray.normal).sum::<Vec3>();
        sum / self.points.len() as f32
    }
}

/// Group points based on a given "shape intersection".
pub fn group_points(
    mut points: Vec<(Entity, RayIntersection)>,
    intersects: impl Fn(RayIntersection, RayIntersection, &[Entity], &[RayIntersection]) -> bool,
) -> Vec<Group> {
    let mut groups = Vec::new();
    while let Some((center_entity, center)) = points.pop() {
        let mut group_entities = Vec::new();
        let mut group_points = Vec::new();
        group_entities.push(center_entity);
        group_points.push(center);

        let mut to_remove = Vec::new();
        for (index, (other_entity, other_ray)) in points.iter().enumerate() {
            if intersects(center, *other_ray, &group_entities, &group_points) {
                group_entities.push(*other_entity);
                group_points.push(*other_ray);
                to_remove.push(index);
            }
        }

        to_remove.sort_by(|a, b| b.cmp(a)); // descending
        for index in to_remove {
            points.swap_remove(index);
        }

        groups.push(Group {
            center_entity,
            center,
            entities: group_entities,
            points: group_points,
        });
    }

    groups
}

pub fn debug_colors(n: usize) -> Vec<Color> {
    let colors = [
        css::RED,
        css::GREEN,
        css::BLUE,
        css::BLACK,
        css::WHITE,
        css::PINK,
        css::SEA_GREEN,
        css::ORANGE,
        css::PURPLE,
    ];

    colors.map(|c| Color::from(c)).to_vec()
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
    _gizmos: &mut RetainedGizmos,
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
            toi = ray.time_of_impact;
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
