use bevy::{
    asset::HandleId,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use bevy_rapier3d::parry::shape::{TriMesh, TypedShape};
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
        let view: ColliderView = (*self).into();
        view.as_meshes()
    }
}

impl<'a> AsMesh for ColliderView<'a> {
    fn as_meshes(&self) -> Vec<(Mesh, Transform)> {
        let mut meshes = Vec::new();
        match self {
            ColliderView::Ball(shape_views::BallView { raw: ball }) => {
                let mesh = Mesh::from(shape::UVSphere {
                    radius: ball.radius,
                    ..default()
                });
                meshes.push((mesh, Transform::default()));
            }
            ColliderView::Cuboid(shape_views::CuboidView { raw: cuboid }) => {
                let dim = cuboid.half_extents * 2.0;
                let mesh = Mesh::from(shape::Box::new(dim.x, dim.y, dim.z));
                meshes.push((mesh, Transform::default()));
            }
            ColliderView::Capsule(shape_views::CapsuleView { raw: capsule }) => {
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
            ColliderView::Segment(segment) => {}
            ColliderView::Triangle(triangle) => {}
            ColliderView::TriMesh(shape_views::TriMeshView { raw: trimesh }) => {
                let mesh = trimesh_to_mesh(trimesh);
                meshes.push((mesh, Transform::default()));
            }
            ColliderView::Polyline(polyline) => {}
            ColliderView::HalfSpace(half_space) => {}
            ColliderView::HeightField(shape_views::HeightFieldView { raw: height_field }) => {
                let (points, indices) = height_field.to_trimesh();
                let trimesh = TriMesh::new(points, indices);
                let mesh = trimesh_to_mesh(&trimesh);
                meshes.push((mesh, Transform::default()));
            }
            ColliderView::Compound(shape_views::CompoundView { raw: compound }) => {
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
            ColliderView::ConvexPolyhedron(shape_views::ConvexPolyhedronView {
                raw: convex_polyhedron,
            }) => {
                let (points, indices) = convex_polyhedron.to_trimesh();
                let trimesh = TriMesh::new(points, indices);
                let mesh = trimesh_to_mesh(&trimesh);
                meshes.push((mesh, Transform::default()));
            }
            ColliderView::Cylinder(shape_views::CylinderView { raw: cylinder }) => {
                let mesh = Mesh::from(shape::Cylinder {
                    radius: cylinder.radius,
                    height: cylinder.half_height * 2.0,
                    ..default()
                });
                meshes.push((mesh, Transform::default()));
            }
            ColliderView::Cone(cone) => {}
            ColliderView::RoundCuboid(round_cuboid) => {}
            ColliderView::RoundTriangle(round_triangle) => {}
            ColliderView::RoundCylinder(round_cylinder) => {}
            ColliderView::RoundCone(round_cone) => {}
            ColliderView::RoundConvexPolyhedron(round_convex_polyhedron) => {}
            _ => {}
        };

        meshes
    }
}

pub fn init_physics_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    rigid_bodies: Query<
        (Entity, Option<&Children>, &Collider),
        (With<RigidBody>, Changed<Collider>),
    >,
    physics_mesh: Query<&PhysicsDebugMesh>,
    names: Query<&Name>,
) {
    for (entity, children, collider) in &rigid_bodies {
        let mut found = false;
        if let Some(children) = children {
            for child in children.iter() {
                if physics_mesh.get(*child).is_ok() {
                    commands.entity(*child).despawn_recursive();
                }
            }
        }

        if found {
            //continue;
        }

        let name = names
            .get(entity)
            .map(|name| name.as_str().to_owned())
            .unwrap_or(format!("{:?}", entity));

        for (mesh, transform) in collider.as_unscaled_typed_shape().as_meshes() {
            let handle = meshes.add(mesh);
            let physics_mesh = commands
                .spawn(PbrBundle {
                    mesh: handle,
                    transform: transform,
                    ..default()
                })
                .insert(PhysicsDebugMesh)
                .insert(Name::new("Physics debug mesh"))
                .id();

            commands
                .entity(entity)
                .insert(Visibility::default())
                .insert(ComputedVisibility::default())
                .add_child(physics_mesh);
        }
    }
}
