use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLines;
use sabi::stage::{NetworkCoreStage, NetworkSimulationAppExt};

#[derive(Debug, Clone, Component)]
pub struct Attach(Entity);

impl Attach {
    pub fn scale(entity: Entity) -> (Attach, AttachScale) {
        (Attach(entity), AttachScale::Instant)
    }

    pub fn rotation(entity: Entity) -> (Attach, AttachRotation) {
        (Attach(entity), AttachRotation::Instant)
    }

    pub fn translation(entity: Entity) -> (Attach, AttachTranslation) {
        (Attach(entity), AttachTranslation::Instant)
    }

    pub fn all(entity: Entity) -> (Attach, AttachTranslation, AttachRotation, AttachScale) {
        (
            Attach(entity),
            AttachTranslation::Instant,
            AttachRotation::Instant,
            AttachScale::Instant,
        )
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Default, Debug, Clone, Component, Reflect, Inspectable)]
#[reflect(Component)]
pub enum AttachTranslation {
    #[default]
    Instant,
    Spring {
        #[inspectable(min = 0.0, max = 10000.0)]
        strength: f32,
        #[inspectable(min = 0.0, max = 1.0)]
        damp_ratio: f32,
    },
}

#[derive(Default, Debug, Clone, Component, Reflect, Inspectable)]
#[reflect(Component)]
pub enum AttachRotation {
    #[default]
    Instant,
    Spring {
        strength: f32,
        damp_ratio: f32,
    },
}

#[derive(Default, Debug, Clone, Component, Reflect, Inspectable)]
#[reflect(Component)]
pub enum AttachScale {
    #[default]
    Instant,
    Spring {
        strength: f32,
        damp_ratio: f32,
    },
}

pub fn simple_harmonic_motion(strength: f32, damping: f32) -> (f32, f32) {
    /*
    w = sqrt(strength / mass)
    x = offset*cos(w*delta) + velocity/w*sin(w*delta)
    v = -w*offset*sin(w*delta) + velocity*cos(w*delta)
    */
    const EPSILON: f32 = 0.0001;
    let frequency = strength.max(0.0);
    let damping = damping.max(0.0);
    //let delta_time = crate::TICK_RATE.as_secs_f32();
    let delta_time = 1.0;

    if damping > 1.0 + EPSILON {
        // over damped
        let za = -frequency * damping;
        let zb = frequency * (damping * damping - 1.0).sqrt();
        let z1 = za - zb;
        let z2 = za + zb;

        let e1 = (z1 * delta_time).exp();
        let e2 = (z2 * delta_time).exp();

        let inv_two_zb = 1.0 / (2.0 * zb);
        let e1_over_two_zb = e1 * inv_two_zb;
        let e2_over_two_zb = e2 * inv_two_zb;

        let z1e1_over_two_zb = z1 * e1_over_two_zb;
        let z2e2_over_two_zb = z2 * e2_over_two_zb;

        let velpos = (z1e1_over_two_zb - z2e2_over_two_zb + e2) * z2;
        let velvel = -z1e1_over_two_zb + z2e2_over_two_zb;
        (velpos, velvel)
    } else if damping < 1.0 - EPSILON {
        let omega_zeta = frequency * damping; // f * d
        let alpha = frequency * (1.0 - damping * damping).sqrt(); // f * sqrt(1 - d^2)

        let exp_term = (-omega_zeta * delta_time).exp(); // e^(-(f * d))
        let cos_term = (alpha * delta_time).cos(); // e^()
        let sin_term = (alpha * delta_time).sin();

        let inv_alpha = 1.0 / alpha;

        let exp_sin = exp_term * sin_term;
        let exp_cos = exp_term * cos_term;
        let exp_omega_zeta_sin_over_alpha = exp_term * omega_zeta * sin_term * inv_alpha;

        let velpos = -exp_sin * alpha - omega_zeta * exp_omega_zeta_sin_over_alpha;
        let velvel = exp_cos - exp_omega_zeta_sin_over_alpha;
        (velpos, velvel)

        /*
                        // under-damped
                        float omegaZeta = angularFrequency * dampingRatio;
                        float alpha     = angularFrequency * sqrtf(1.0f - dampingRatio*dampingRatio);

                        float expTerm = expf( -omegaZeta * deltaTime );
                        float cosTerm = cosf( alpha * deltaTime );
                        float sinTerm = sinf( alpha * deltaTime );

        (e^{-dtw}t(qtcos(qt)(dtw+v(0))-sin(qt)(dtw+v(0)+q^2tx(0)))dq)/q^2+(e^(-dtw)(qcos(qt)(tv(0)+dtw(t-x(0))+x(0))-sin(qt)(d^2t^2w^2+dtw(-2+v(0))-v(0)+q^2tx(0)))dt)/q-(de^(-dtw)t^2(sin(qt)(-1+dtw+v(0))+qcos(qt)x(0))dw)/q

                d(-e^(-dtw)(sin(qt)((dw(dtw+v(0)))/q+qx(0))+cos(qt)(-dtw-v(0)+dwx(0))))=-e^(-dtw)(tsin(qt)(v(0)+dw(t-x(0)))+sin(qt)(-(dw(dtw+v(0)))/q^2+x(0))+tcos(qt)((dw(dtw+v(0)))/q+qx(0)))dq+(e^(-dtw)(sin(qt)(d^3tw^3+d^2w^2(-1+v(0))-q^2v(0)-dq^2w(t-2x(0)))+qcos(qt)(d(w-2wv(0))-q^2x(0)+d^2w^2(-2t+x(0))))dt)/q+(de^(-dtw)(sin(qt)(d^2t^2w^2+dtw(-2+v(0))-v(0)+q^2tx(0))-qcos(qt)(dt^2w+x(0)+t(-1+v(0)-dwx(0))))dw)/q
                        float invAlpha = 1.0f / alpha;

                        float expSin = expTerm*sinTerm;
                        float expCos = expTerm*cosTerm;
                        float expOmegaZetaSin_Over_Alpha = expTerm*omegaZeta*sinTerm*invAlpha;

                        pOutParams->m_posPosCoef = expCos + expOmegaZetaSin_Over_Alpha;
                        pOutParams->m_posVelCoef = expSin*invAlpha;

                        pOutParams->m_velPosCoef = -expSin*alpha - omegaZeta*expOmegaZetaSin_Over_Alpha;
                        pOutParams->m_velVelCoef =  expCos - expOmegaZetaSin_Over_Alpha;
                        */
    } else {
        let exp_term = (-frequency * delta_time).exp(); // e^(-f)
        let time_exp = (exp_term * delta_time); // e^(-f) * d
        let time_exp_freq = time_exp * frequency; // e^(-f) * f

        // -f * (e^(-f) * f)
        // -(e^(-f) * f) + e^(-f)

        let velpos = -frequency * time_exp_freq;
        let velvel = -time_exp_freq + exp_term;
        (velpos, velvel)
    }
}

#[derive(Debug, Clone, Component)]
pub struct PreviousTransform(pub Transform);

pub fn velocity_nonphysics(mut velocities: Query<(&mut Transform, &Velocity), Without<RigidBody>>) {
    for (mut transform, velocity) in &mut velocities {
        transform.translation += velocity.linvel * crate::TICK_RATE.as_secs_f32();
    }
}

pub fn damped_spring(position: Vec3, desired: Vec3) {}

pub fn update_attach(
    invalid_attachers: Query<Entity, (With<Attach>, With<Parent>)>,
    mut attachers: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut ExternalImpulse,
            &mut ExternalForce,
            &ReadMassProperties,
            &Attach,
            Option<&AttachTranslation>,
            Option<&AttachRotation>,
            Option<&AttachScale>,
        ),
        Or<(
            With<AttachTranslation>,
            With<AttachRotation>,
            With<AttachScale>,
        )>,
    >,
    global: Query<&GlobalTransform>,
    mut lines: ResMut<DebugLines>,
) {
    for invalid_attacher in &invalid_attachers {
        info!(
            "attacher is invalid, cannot use the transform hierarchy: {:?}",
            invalid_attacher
        );
    }

    for (
        entity,
        mut transform,
        mut velocity,
        mut external_impulse,
        mut external_force,
        mass_properties,
        attach,
        translation,
        rotation,
        scale,
    ) in &mut attachers
    {
        if let Ok(global) = global.get(attach.get()) {
            let global_transform = global.compute_transform();
            match translation {
                Some(AttachTranslation::Instant) => {
                    transform.translation = global_transform.translation;
                }
                Some(&AttachTranslation::Spring {
                    strength,
                    damp_ratio,
                }) => {
                    let strength = strength.max(0.0);
                    let damp_ratio = damp_ratio.max(0.0);
                    let mass = mass_properties.0.mass;

                    if mass <= 0.0 || strength <= 0.0 {
                        continue;
                    }

                    let critical_damping = 2.0 * (mass * strength).sqrt();
                    let damp_coefficient = damp_ratio * critical_damping;

                    let offset = transform.translation - global_transform.translation;
                    let offset_force = -strength * offset;
                    //let new_velocity = velocity.linvel + offset_force;

                    //let spring_force = offset_force + damp_force;

                    /*
                                       dbg!(
                                           damp_ratio,
                                           critical_damping,
                                           damp_coefficient,
                                           velocity.linvel,
                                           offset_force,
                                           damp_force
                                       );
                    */

                    velocity.linvel += offset_force;
                    let vel = velocity.linvel
                        + velocity
                            .angvel
                            .cross(Vec3::ZERO - mass_properties.0.local_center_of_mass);

                    let damp_force = -damp_coefficient * vel;
                    velocity.linvel += damp_force;
                    //external_force.force = spring_force;

                    lines.line_colored(
                        transform.translation,
                        transform.translation + external_force.force,
                        crate::TICK_RATE.as_secs_f32(),
                        Color::YELLOW,
                    );

                    /*
                                       lines.line_colored(
                                           transform.translation,
                                           transform.translation + impulse,
                                           crate::TICK_RATE.as_secs_f32(),
                                           Color::BLUE,
                                       );
                    */
                }
                _ => {}
            }

            if rotation.is_some() {
                transform.rotation = global_transform.rotation;
                velocity.angvel = Vec3::ZERO;
            }

            if scale.is_some() {
                transform.scale = global_transform.scale;
            }
        }
    }
}

pub struct AttachPlugin;

impl Plugin for AttachPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttachTranslation>()
            .register_type::<AttachRotation>()
            .register_type::<AttachScale>();

        app.register_inspectable::<AttachTranslation>()
            .register_inspectable::<AttachRotation>()
            .register_inspectable::<AttachScale>();

        app.add_network_system(velocity_nonphysics.label("velocity_nonphysics"));
        app.add_network_system(update_attach.label("update_attach"));
    }
}
