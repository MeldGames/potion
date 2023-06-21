use bevy::{
    asset::HandleId,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use bevy_rapier3d::parry::shape::TypedShape;
use bevy_rapier3d::prelude::*;

#[derive(Component, Copy, Clone, Debug, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct PhysicsDebugMesh;

pub fn init_physics_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    rigid_bodies: Query<(Entity, Option<&Children>, &Collider), With<RigidBody>>,
    physics_mesh: Query<&PhysicsDebugMesh>,
    names: Query<&Name>,
) {
    for (entity, children, collider) in &rigid_bodies {
        let mut found = false;
        if let Some(children) = children {
            for child in children.iter() {
                if physics_mesh.get(*child).is_ok() {
                    found = true;
                }
            }
        }

        if found {
            continue;
        }

        let name = names
            .get(entity)
            .map(|name| name.as_str().to_owned())
            .unwrap_or(format!("{:?}", entity));
        info!("adding physics debug mesh to {:?}", name);

        let as_mesh = match collider.raw.as_typed_shape() {
            TypedShape::Ball(ball) => {
                let mesh = meshes.add(Mesh::from(shape::UVSphere {
                    radius: ball.radius,
                    ..default()
                }));
                Some((mesh, Vec3::ZERO))
            }
            TypedShape::Cuboid(cuboid) => {
                let dim = cuboid.half_extents * 2.0;
                let mesh = meshes.add(Mesh::from(shape::Box::new(dim.x, dim.y, dim.z)));
                Some((mesh, Vec3::ZERO))
            }
            TypedShape::Capsule(capsule) => {
                let a: Vec3 = capsule.segment.a.into();
                let b: Vec3 = capsule.segment.b.into();
                let midpoint = a * 0.5 + b * 0.5;
                let length = (a - b).length();
                let mesh = meshes.add(Mesh::from(shape::Capsule {
                    depth: length,
                    radius: capsule.radius,
                    ..default()
                }));

                Some((mesh, midpoint))
            }
            TypedShape::Segment(segment) => {
                info!("segment: {:?}", segment);
                None
            }
            TypedShape::Triangle(triangle) => {
                info!("triangle: {:?}", triangle);
                None
            }
            TypedShape::TriMesh(trimesh) => {
                info!("trimesh");
                None
            }
            TypedShape::Polyline(polyline) => {
                info!("polyline");
                None
            }
            TypedShape::HalfSpace(half_space) => {
                info!("half_space: {:?}", half_space);
                None
            }
            TypedShape::HeightField(height_field) => {
                info!("height_field: {:?}", height_field);
                None
            }
            TypedShape::Compound(compound) => {
                info!("compound");
                None
            }
            TypedShape::ConvexPolyhedron(convex_polyhedron) => {
                info!("convex_polyhedron: {:?}", convex_polyhedron);
                None
            }
            TypedShape::Cylinder(cylinder) => {
                info!("cylinder: {:?}", cylinder);
                None
            }
            TypedShape::Cone(cone) => {
                info!("cone: {:?}", cone);
                None
            }
            TypedShape::RoundCuboid(round_cuboid) => {
                info!("round_cuboid: {:?}", round_cuboid);
                None
            }
            TypedShape::RoundTriangle(round_triangle) => {
                info!("round_triangle: {:?}", round_triangle);
                None
            }
            TypedShape::RoundCylinder(round_cylinder) => {
                info!("round_cylinder: {:?}", round_cylinder);
                None
            }
            TypedShape::RoundCone(round_cone) => {
                info!("round_cone: {:?}", round_cone);
                None
            }
            TypedShape::RoundConvexPolyhedron(round_convex_polyhedron) => {
                info!("round_convex_polyhedron: {:?}", round_convex_polyhedron);
                None
            }
            TypedShape::Custom(id) => {
                info!("custom: {:?}", id);
                None
            }
            _ => {
                info!("unknown shape");
                None
            }
        };

        if let Some((mesh, translation)) = as_mesh {
            let physics_mesh = commands
                .spawn(PbrBundle {
                    mesh: mesh,
                    transform: Transform {
                        translation,
                        ..default()
                    },
                    ..default()
                })
                .insert(PhysicsDebugMesh)
                .id();

            commands
                .entity(entity)
                .insert(Visibility::default())
                .insert(ComputedVisibility::default())
                .add_child(physics_mesh);
        }
    }
}
