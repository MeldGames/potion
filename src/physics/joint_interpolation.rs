use crate::prelude::*;

#[derive(Default, Component)]
pub struct JointInterpolation {
    pub start: GenericJoint,
    pub end: GenericJoint,
    /// Over how many seconds should this occur.
    pub over: f32,

    /// Internal tracker of how far we've gone through interpolation.
    ///
    /// 0..1
    pub time: f32,
}

impl JointInterpolation {
    pub fn tick(&mut self, delta_time: f32) {
        self.time += (1.0 / self.over) * delta_time;
        self.time = self.time.clamp(0., 1.0);
    }

    pub fn lerp_inplace(&self, joint: &mut GenericJoint) {
        joint.set_local_anchor1(
            self.start
                .local_anchor1()
                .lerp(self.end.local_anchor1(), self.time),
        );
        joint.set_local_anchor2(
            self.start
                .local_anchor2()
                .lerp(self.end.local_anchor2(), self.time),
        );
    }
}

pub struct JointInterpolationPlugin;
impl Plugin for JointInterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, joint_interpolation);
    }
}

pub fn joint_interpolation(
    ctx: Res<RapierContext>,
    mut joints: Query<(&mut ImpulseJoint, &mut JointInterpolation)>,
) {
    let dt = ctx.integration_parameters.dt;

    for (mut joint, mut interp) in &mut joints {
        interp.tick(dt);
        interp.lerp_inplace(joint.data.as_mut());
    }
}
