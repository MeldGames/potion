use bevy::{
    asset::HandleId,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use bevy_rapier3d::parry::shape::{TypedShape, TriMesh};
use bevy_rapier3d::prelude::*;

#[derive(Component, Copy, Clone, Debug, Reflect, FromReflect, Default)]
#[reflect(Component)]
pub struct PhysicsDebugMesh;

pub trait AsMesh {
    fn as_meshes(&self) -> Vec<(Mesh, Transform)>;
}

pub fn trimesh_to_mesh(trimesh: &TriMesh) -> Mesh {
    let points = trimesh.vertices();
    let indices = trimesh.indices();
    let points: Vec<[f32; 3]> = points
        .iter()
        .map(|point| [point.x, point.y, point.z])
        .collect();
    let indices: Vec<u32> = indices.iter().flatten().cloned().collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();
    mesh
}

impl<'a> AsMesh for TypedShape<'a> {
    fn as_meshes(&self) -> Vec<(Mesh, Transform)> {
        let mut meshes = Vec::new();
        match self {
            TypedShape::Ball(ball) => {
                let mesh = Mesh::from(shape::UVSphere {
                    radius: ball.radius,
                    ..default()
                });
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Cuboid(cuboid) => {
                let dim = cuboid.half_extents * 2.0;
                let mesh = Mesh::from(shape::Box::new(dim.x, dim.y, dim.z));
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Capsule(capsule) => {
                let a: Vec3 = capsule.segment.a.into();
                let b: Vec3 = capsule.segment.b.into();
                let midpoint = a * 0.5 + b * 0.5;
                let length = (a - b).length();
                let mesh = Mesh::from(shape::Capsule {
                    depth: length,
                    radius: capsule.radius,
                    ..default()
                });
                meshes.push((
                    mesh,
                    Transform {
                        translation: midpoint,
                        ..default()
                    },
                ));
            }
            TypedShape::Segment(segment) => {
                info!("segment: {:?}", segment);
            }
            TypedShape::Triangle(triangle) => {
                info!("triangle: {:?}", triangle);
            }
            TypedShape::TriMesh(trimesh) => {
                let mesh = trimesh_to_mesh(trimesh);
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Polyline(polyline) => {
                info!("polyline");
            }
            TypedShape::HalfSpace(half_space) => {
                info!("half_space: {:?}", half_space);
            }
            TypedShape::HeightField(height_field) => {
                info!("height_field: {:?}", height_field);
            }
            TypedShape::Compound(compound) => {
                for (isometry, shape) in compound.shapes() {
                    let compound_transform = Transform {
                        translation: isometry.translation.into(),
                        rotation: isometry.rotation.into(),
                        scale: Vec3::ONE,
                    };

                    let typed_shape = shape.as_typed_shape();
                    for (mesh, transform) in typed_shape.as_meshes() {
                        let transform = compound_transform * transform;
                        meshes.push((mesh, transform));
                    }
                }
            }
            TypedShape::ConvexPolyhedron(convex_polyhedron) => {
                let (points, indices) = convex_polyhedron.to_trimesh();
                let trimesh = TriMesh::new(points, indices);
                let mesh = trimesh_to_mesh(&trimesh);
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Cylinder(cylinder) => {
                let mesh = Mesh::from(shape::Cylinder {
                    radius: cylinder.radius,
                    height: cylinder.half_height * 2.0,
                    ..default()
                });
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Cone(cone) => {
                info!("cone: {:?}", cone);
            }
            TypedShape::RoundCuboid(round_cuboid) => {
                info!("round_cuboid: {:?}", round_cuboid);
            }
            TypedShape::RoundTriangle(round_triangle) => {
                info!("round_triangle: {:?}", round_triangle);
            }
            TypedShape::RoundCylinder(round_cylinder) => {
                info!("round_cylinder: {:?}", round_cylinder);
            }
            TypedShape::RoundCone(round_cone) => {
                info!("round_cone: {:?}", round_cone);
            }
            TypedShape::RoundConvexPolyhedron(round_convex_polyhedron) => {
                info!("round_convex_polyhedron");
            }
            TypedShape::Custom(id) => {
                info!("custom: {:?}", id);
            }
            _ => {
                info!("unknown shape");
            }
        };

        meshes
    }
}

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

        for (mesh, transform) in collider.raw.as_typed_shape().as_meshes() {
            let handle = meshes.add(mesh);
            let physics_mesh = commands
                .spawn(PbrBundle {
                    mesh: handle,
                    transform: transform,
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
