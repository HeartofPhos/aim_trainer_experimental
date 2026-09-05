use crate::input::InputAggregator;
use aim_core::logic::challenge::ChallengeValue;
pub use aim_core::{
    AimCore, Camera, Input, Render,
    logic::{
        challenge::Challenge,
        health::Health,
        level::{BrushDef, PrimitiveCache},
        shape::Shape,
    },
};
use bevy::prelude::*;
use std::time::Duration;

pub mod prelude {
    pub use super::AimBridge;
    pub use aim_core::{
        AimCore, Camera, Input, Render,
        logic::{
            challenge::Challenge,
            health::Health,
            level::{BrushDef, PrimitiveCache},
            shape::Shape,
        },
    };
}

mod input;

pub struct AimBridge {
    timestep: Duration,
    elapsed: Duration,
    accumulator: Duration,
    input_aggregator: InputAggregator,
    app: App,
}

impl AimBridge {
    pub fn new(aim_core: AimCore) -> AimBridge {
        Self {
            timestep: aim_core.timestep,
            elapsed: Default::default(),
            accumulator: Default::default(),
            input_aggregator: Default::default(),
            app: aim_core.build(),
        }
    }

    pub fn primitive_cache(&self) -> &PrimitiveCache {
        self.app.world().resource::<PrimitiveCache>()
    }

    pub fn level_brushes(&self, mut f: impl FnMut(&schema::BrushDef, &schema::BrushTransform)) {
        let mut query = self
            .app
            .world()
            .try_query::<(&BrushDef, &Transform)>()
            .unwrap();

        for (brush_def, transform) in query.iter(self.app.world()) {
            f(
                brush_def,
                &schema::BrushTransform {
                    translation: transform.translation,
                    rotation: transform.rotation,
                    scale: transform.scale,
                },
            );
        }
    }

    pub fn update(&mut self, delta_time: Duration, input: Input) {
        self.input_aggregator.push(input);

        self.accumulator += delta_time;
        while self.accumulator > self.timestep {
            let input = self.input_aggregator.take();
            self.app.insert_resource(input);
            self.app.update();

            self.accumulator -= self.timestep;
            self.elapsed += self.timestep;
        }
    }

    pub fn camera(&self) -> Result<Transform> {
        let transform = self
            .app
            .world()
            .try_query_filtered::<&GlobalTransform, With<Camera>>()
            .ok_or("failed query")?
            .single(self.app.world())?;

        Ok((*transform).into())
    }

    pub fn shapes(&self, mut f: impl FnMut(Transform, Shape, Render, Option<Health>)) {
        let mut query = self
            .app
            .world()
            .try_query_filtered::<(&GlobalTransform, &Shape, &Render, Option<&Health>), ()>()
            .unwrap();

        for (transform, shape, render, health) in query.iter(self.app.world()) {
            f((*transform).into(), *shape, *render, health.copied());
        }
    }

    pub fn challenge(&self) -> ChallengeValue {
        self.app.world().resource::<Challenge>().value()
    }
}
