use bevy_rapier3d::{
    prelude::shape_views::*,
    rapier::prelude::{SharedShape, TypedShape},
};

use crate::prelude::*;

pub fn split_compound(
    mut commands: Commands,
    ctx: Res<RapierContext>,
    colliders: Query<(Entity, &Collider), Changed<Collider>>,
) {
    for (entity, collider) in &colliders {
        let ColliderView::Compound(compound) = collider.as_unscaled_typed_shape() else { continue };
        let Some(parent) = ctx.collider_parent(entity) else { continue };

        commands
            .entity(entity)
            .remove::<Collider>()
            .with_children(|children| {
                for (translation, rotation, shape) in compound.shapes() {
                    let shape: SharedShape = match shape {
                        ColliderView::Ball(BallView { raw }) => SharedShape::new(raw.clone()),
                        ColliderView::Cuboid(CuboidView { raw }) => SharedShape::new(raw.clone()),
                        ColliderView::Capsule(CapsuleView { raw }) => SharedShape::new(raw.clone()),
                        ColliderView::Segment(SegmentView { raw }) => SharedShape::new(raw.clone()),
                        ColliderView::Triangle(TriangleView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::TriMesh(TriMeshView { raw }) => SharedShape::new(raw.clone()),
                        ColliderView::Polyline(PolylineView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::HalfSpace(HalfSpaceView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::HeightField(HeightFieldView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::Compound(CompoundView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::ConvexPolyhedron(ConvexPolyhedronView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::Cylinder(CylinderView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::Cone(ConeView { raw }) => SharedShape::new(raw.clone()),
                        ColliderView::RoundCuboid(RoundCuboidView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::RoundTriangle(RoundTriangleView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::RoundCylinder(RoundCylinderView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::RoundCone(RoundConeView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                        ColliderView::RoundConvexPolyhedron(RoundConvexPolyhedronView { raw }) => {
                            SharedShape::new(raw.clone())
                        }
                    };
                    children
                        .spawn(SpatialBundle {
                            transform: Transform {
                                translation,
                                rotation,
                                ..default()
                            },
                            ..default()
                        })
                        .insert(ColliderBundle::collider(shape));
                }
            });
    }
}
