use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Previous<C: Component + Clone>(pub C);

pub fn previous<C: Component + Clone>(mut components: Query<(&mut Previous<C>, &C)>) {
    for (mut previous, current) in &mut components {
        previous.0 = current.clone();
    }
}
