use bevy::{ecs::system::SystemParam, prelude::*};

crate::relationships! {
    pub
    crate::relationship!(pub, one_one, TargeterOf, [], Targeter, []);
}

#[derive(SystemParam)]
pub struct UseTargeter<'w, 's> {
    targeter_query: Query<'w, 's, &'static Targeter>,
    transform_query: Query<'w, 's, &'static GlobalTransform>,
}

impl UseTargeter<'_, '_> {
    pub fn get_transform(&self, source: Entity) -> Result<&GlobalTransform> {
        let targeter = self.targeter_query.get(source)?;
        let transform = self.transform_query.get(targeter.entity())?;

        Ok(transform)
    }

    pub fn get_ray(&self, source: Entity, direction: Dir3) -> Result<Ray3d> {
        let transform = self.get_transform(source)?;

        let ray = Ray3d {
            origin: transform.translation(),
            direction: transform.rotation() * direction,
        };

        Ok(ray)
    }
}
