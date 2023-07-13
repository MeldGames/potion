use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;
use bevy_rapier3d::{
    prelude::*,
    rapier::dynamics::{JointAxesMask, JointAxis},
};

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Slot {
    /// Entity this slot contains.
    #[reflect(default)]
    pub containing: Option<Entity>,
}

#[derive(Debug, Clone, Bundle)]
pub struct SlotBundle {
    pub slot: Slot,
    pub settings: SlotSettings,
    pub grace: SlotGracePeriod,
}

#[derive(Default, Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct SlotSettings(pub springy::Spring);

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Component, Reflect)]
#[reflect(Component)]
pub enum Slottable {
    #[default]
    Free,

    Slotted,
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct SlotGracePeriod(Timer);

impl Default for SlotGracePeriod {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(1), TimerMode::Once))
    }
}

#[derive(Debug, Clone, Component)]
pub struct SlotDeposit {
    pub slots: Vec<Entity>,
    pub attempting: VecDeque<Entity>,
}

impl SlotDeposit {
    pub fn new(slots: Vec<Entity>) -> Self {
        Self {
            slots,
            attempting: VecDeque::new(),
        }
    }

    pub fn contains(&self, _entity: Entity) -> Option<usize> {
        self.attempting
            .iter()
            .enumerate()
            .find(|(_index, entity)| entity == entity)
            .map(|(index, _)| index)
    }

    pub fn attempt(&mut self, entity: Entity) {
        if let None = self.contains(entity) {
            self.attempting.push_back(entity);
        }
    }

    pub fn stop_attempt(&mut self, entity: Entity) {
        if let Some(index) = self.contains(entity) {
            let removed = self.attempting.remove(index);
            assert_eq!(removed, Some(entity));
        }
    }

    pub fn pop_attempt(&mut self) -> Option<Entity> {
        self.attempting.pop_front()
    }
}

pub fn pending_slot(
    names: Query<DebugName>,
    mut slotters: Query<(Entity, &mut SlotDeposit)>,
    slottable: Query<(Entity, &Slottable)>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        let ((_slotter_entity, mut slotter), (ingredient_entity, slottable), colliding) =
            match collision_event {
                &CollisionEvent::Started(collider1, collider2, _flags) => {
                    let (slotter, potential) = if let Ok(slotter) = slotters.get_mut(collider1) {
                        (slotter, collider2)
                    } else if let Ok(slotter) = slotters.get_mut(collider2) {
                        (slotter, collider1)
                    } else {
                        continue;
                    };

                    if let Ok(ingredient) = slottable.get(potential) {
                        (slotter, ingredient, true)
                    } else {
                        continue;
                    }
                }
                &CollisionEvent::Stopped(collider1, collider2, _flags) => {
                    let (slotter, potential) = if let Ok(slotter) = slotters.get_mut(collider1) {
                        (slotter, collider2)
                    } else if let Ok(slotter) = slotters.get_mut(collider2) {
                        (slotter, collider1)
                    } else {
                        continue;
                    };

                    if let Ok(ingredient) = slottable.get(potential) {
                        (slotter, ingredient, false)
                    } else {
                        continue;
                    }
                }
            };

        if *slottable == Slottable::Slotted {
            continue;
        }

        if colliding {
            slotter.attempt(ingredient_entity);
        } else {
            slotter.stop_attempt(ingredient_entity);
        }

        info!(
            "attempting: {:?}",
            slotter
                .attempting
                .iter()
                .map(|entity| names.get(*entity).unwrap())
                .collect::<Vec<_>>()
        );
    }
}

pub fn tick_grace_period(mut slots: Query<&mut SlotGracePeriod>) {
    for mut period in &mut slots {
        period.0.tick(crate::TICK_RATE);
    }
}

pub fn insert_slot(
    mut slotted: Query<&mut Slottable>,
    mut slots: Query<(&mut Slot, &mut SlotGracePeriod)>,
    mut deposits: Query<&mut SlotDeposit>,
    names: Query<DebugName>,
) {
    for mut deposit in &mut deposits {
        if deposit.slots.len() == 0 {
            warn!("no slots specified in slot deposit");
            continue;
        }

        let SlotDeposit {
            slots: deposit_slots,
            attempting,
        } = deposit.as_mut();

        if attempting.len() == 0 {
            continue;
        }

        for slot_entity in &*deposit_slots {
            let Ok((mut slot, mut grace_period)) = slots.get_mut(*slot_entity) else { continue };
            if slot.containing.is_none() {
                while let Some(next_item) = attempting.pop_front() {
                    let Ok(mut slottable) = slotted.get_mut(next_item) else { continue };
                    if *slottable == Slottable::Free {
                        info!("slotting {:?}", names.get(next_item).unwrap());
                        slot.containing = Some(next_item);

                        grace_period.0 = Timer::new(Duration::from_secs(1), TimerMode::Once);
                        *slottable = Slottable::Slotted;
                        break;
                    }
                }
            }
        }
    }
}

pub fn slot_joints(
    mut commands: Commands,
    slots: Query<(Entity, &Slot)>,
    joints: Query<(&ImpulseJoint, &RapierImpulseJointHandle)>,
    //rapier: Res<RapierContext>,
    //names: Query<&Name>,
) {
    for (entity, slot) in &slots {
        match slot.containing {
            Some(item) => {
                if let Ok((_joint, _handle)) = joints.get(entity) {
                    /*
                    let name = names
                        .get(entity)
                        .map(|name| name.as_str())
                        .unwrap_or("blah");

                    let joint = rapier.impulse_joints.get(handle.0).unwrap();
                    info!(
                        "slot: {:?}, impulse: {:?}, motors: {:?}",
                        name,
                        joint.impulses,
                        joint
                            .data
                            .motors
                            .iter()
                            .map(|motor| motor.impulse)
                            .collect::<Vec<_>>()
                    );
                    */
                } else {
                    let strength = 5000.0;
                    let damping = 5.0;
                    /*
                                    let mut slot_joint = FixedJointBuilder::new()
                                        .build();
                    */
                    let slot_joint = GenericJointBuilder::new(JointAxesMask::empty())
                        .motor_position(JointAxis::X, 0.0, strength, damping)
                        .motor_max_force(JointAxis::X, 300.0)
                        .motor_position(JointAxis::Y, 0.0, strength, damping)
                        .motor_max_force(JointAxis::Y, 300.0)
                        .motor_position(JointAxis::Z, 0.0, strength, damping)
                        .motor_max_force(JointAxis::Z, 300.0)
                        .motor_position(JointAxis::AngX, 0.0, strength, damping)
                        .motor_position(JointAxis::AngY, 0.0, strength, damping)
                        .motor_position(JointAxis::AngZ, 0.0, strength, damping)
                        .build();
                    commands
                        .entity(entity)
                        .insert(ImpulseJoint::new(item, slot_joint));
                }
            }
            None => {
                commands.entity(entity).remove::<ImpulseJoint>();
            }
        }
    }
}

pub struct SlotPlugin;
impl Plugin for SlotPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Slot>()
            .register_type::<Option<Entity>>()
            .register_type::<springy::Spring>()
            .register_type::<bevy::time::TimerMode>()
            .register_type::<SlotSettings>();

        app.add_systems(
            FixedUpdate,
            (
                pending_slot,
                insert_slot.after(pending_slot),
                tick_grace_period.before(insert_slot),
                slot_joints.after(insert_slot),
            )
                .before(PhysicsSet::SyncBackend),
        );
    }
}
