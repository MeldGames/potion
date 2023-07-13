use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use crate::prelude::*;
use bevy_rapier3d::parry::shape::{TriMesh, TypedShape};
use bevy_rapier3d::prelude::*;

#[derive(Component, Copy, Clone, Debug, Reflect, Default)]
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
            ColliderView::Segment(_segment) => {}
            ColliderView::Triangle(_triangle) => {}
            ColliderView::TriMesh(shape_views::TriMeshView { raw: trimesh }) => {
                let mesh = trimesh_to_mesh(trimesh);
                meshes.push((mesh, Transform::default()));
            }
            ColliderView::Polyline(_polyline) => {}
            ColliderView::HalfSpace(_half_space) => {}
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
            ColliderView::Cone(_cone) => {}
            ColliderView::RoundCuboid(_round_cuboid) => {}
            ColliderView::RoundTriangle(_round_triangle) => {}
            ColliderView::RoundCylinder(_round_cylinder) => {}
            ColliderView::RoundCone(_round_cone) => {}
            ColliderView::RoundConvexPolyhedron(_round_convex_polyhedron) => {}
        };

        meshes
    }
}

/// If the collider has changed, then produce a new debug mesh for it.
pub fn init_physics_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    colliders: Query<(Entity, &Collider), Changed<Collider>>,
    childrens: Query<&Children>,
    physics_mesh: Query<&PhysicsDebugMesh>,
    mut removed: RemovedComponents<Collider>,
) {
    for entity in removed.iter() {
        if let Ok(children) = childrens.get(entity) {
            for child in children.iter() {
                if physics_mesh.contains(*child) {
                    commands.entity(*child).despawn_recursive();
                }
            }
        }
    }

    for (entity, collider) in &colliders {
        if let Ok(children) = childrens.get(entity) {
            for child in children.iter() {
                if physics_mesh.contains(*child) {
                    commands.entity(*child).despawn_recursive();
                }
            }
        }

        for (mesh, transform) in collider.as_unscaled_typed_shape().as_meshes() {
            let handle = meshes.add(mesh);
            let physics_mesh = commands
                .spawn(PbrBundle {
                    mesh: handle,
                    transform: transform,
                    ..default()
                })
                .insert(PhysicsDebugMesh)
                .insert(DebugVisible)
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
