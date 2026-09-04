use crate::logic::weapon::{ShotDone, ShotResult};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(on_shot_done);
}

pub fn health_bundle(health: f32) -> impl Bundle {
    Health {
        current: health,
        total: health,
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct Health {
    total: f32,
    current: f32,
}

impl Health {
    pub fn ratio(&self) -> f32 {
        let ratio = self.current / self.total;
        if ratio.is_nan() { 1.0 } else { ratio }
    }
}

#[derive(EntityEvent)]
pub struct Killed {
    pub entity: Entity,
    pub by: Option<Entity>,
}

fn on_shot_done(shot_done: On<ShotDone>, mut commands: Commands, mut query: Query<&mut Health>) {
    let ShotResult::HitTarget(target) = shot_done.result else {
        return;
    };

    let Ok(mut health) = query.get_mut(target) else {
        return;
    };

    health.current -= shot_done.shot_info.damage;

    // TODO move to damage trigger?
    if health.current <= 0.0 {
        commands.trigger(Killed {
            entity: target,
            by: Some(shot_done.shooter),
        });
        commands.entity(target).despawn();
    }
}
