use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Resource, Default, Clone, Debug)]
pub struct RetainedGizmos {
    pub spheres: Vec<(f32, Vec3, Quat, f32, Color)>,
    pub cuboids: Vec<(f32, Transform, Color)>,
    pub lines: Vec<(f32, Vec3, Vec3, Color)>,
}

impl RetainedGizmos {
    pub fn sphere(&mut self, time: f32, position: Vec3, rotation: Quat, radius: f32, color: Color) {
        self.spheres.push((time, position, rotation, radius, color));
    }

    pub fn cuboid(&mut self, time: f32, transform: Transform, color: Color) {
        self.cuboids.push((time, transform, color));
    }

    pub fn line(&mut self, time: f32, start: Vec3, end: Vec3, color: Color) {
        self.lines.push((time, start, end, color));
    }

    pub fn ray(&mut self, time: f32, start: Vec3, dir: Vec3, color: Color) {
        self.lines.push((time, start, start + dir, color));
    }

    pub fn apply(&self, gizmos: &mut Gizmos) {
        for (_, position, rotation, radius, color) in &self.spheres {
            gizmos.sphere(*position, *rotation, *radius, *color);
        }

        for (_, cuboid, color) in &self.cuboids {
            gizmos.cuboid(*cuboid, *color);
        }

        for (_, start, end, color) in &self.lines {
            gizmos.line(*start, *end, *color);
        }
    }

    pub fn tick(&mut self, dt: f32) {
        for (ref mut timer, _, _, _, _) in &mut self.spheres {
            *timer -= dt;
        }

        for (ref mut timer, _, _) in &mut self.cuboids {
            *timer -= dt;
        }

        for (ref mut timer, _, _, _) in &mut self.lines {
            *timer -= dt;
        }

        self.filter();
    }

    pub fn filter(&mut self) {
        self.spheres = self
            .spheres
            .iter()
            .cloned()
            .filter(|(timer, ..)| *timer > 0.0)
            .collect();
        self.cuboids = self
            .cuboids
            .iter()
            .cloned()
            .filter(|(timer, ..)| *timer > 0.0)
            .collect();
        self.lines = self
            .lines
            .iter()
            .cloned()
            .filter(|(timer, ..)| *timer > 0.0)
            .collect();
    }
}

pub struct RetainedGizmoPlugin;
impl Plugin for RetainedGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RetainedGizmos::default());
        app.add_systems(PostUpdate, retained_gizmos);
    }
}

pub fn retained_gizmos(
    ctx: Res<RapierContext>,
    mut retained: ResMut<RetainedGizmos>,
    mut gizmos: Gizmos,
) {
    retained.apply(&mut gizmos);
    retained.tick(ctx.integration_parameters.dt);
}
