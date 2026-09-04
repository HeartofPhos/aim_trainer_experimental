use crate::{
    logic::{CharacterControllerSet, TimeFactor},
    utils::Direction,
};
use avian3d::{
    math::{AdjustPrecision, AsF32, Dir, Scalar, Vector},
    prelude::*,
};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (move_and_slide, update_grounded)
            .chain()
            .in_set(CharacterControllerSet),
    );
}

#[derive(Component, Default, Clone, Copy)]
#[require(
    RigidBody::Kinematic,
    CustomPositionIntegration,
    // We don't want to impart speculative collision impulses in this case
    SpeculativeMargin(0.0)
)]
pub struct CharacterController;

/// Component for configuring ground detection for a character controller.
#[derive(Component, Clone)]
pub struct GroundDetection {
    /// The maximum angle (in radians) where a surface is considered ground/ceiling
    /// relative to the up-direction. Outside of this angle, surfaces are considered walls.
    ///
    /// **Default**: 30 degrees (π / 6 radians)
    pub max_angle: Scalar,
    /// The maximum distance for ground detection.
    pub max_distance: Scalar,
    /// The shape cast collider used for ground detection.
    pub cast_shape: Option<Collider>,
}

impl Default for GroundDetection {
    fn default() -> Self {
        Self {
            max_angle: avian3d::math::PI / 6.0,
            max_distance: 0.2,
            cast_shape: None,
        }
    }
}

/// A marker component indicating that an entity is on a surface that is considered
/// ground, meaning the steepness is less than [`GroundDetection::max_angle`].
///
/// Characters that are grounded can jump, and do not slide down slopes.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// A component containing information about the current collisions for a character controller.
///
/// This is used to apply forces to dynamic rigid bodies hit by the character.
#[derive(Component, Default, Deref)]
pub struct CharacterCollisions(Vec<CharacterCollision>);

/// Information about a collision between a character controller and another collider.
pub struct CharacterCollision {
    /// The collider that was hit by the character.
    pub collider: Entity,
    /// The point of contact in world space.
    pub point: Vector,
    /// The normal of the contact surface, pointing away from the character.
    pub normal: Dir3,
    /// The velocity of the character at the point of contact.
    pub character_velocity: Vector,
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &GroundDetection,
        Option<&Collider>,
        &CollisionLayers,
        &GlobalTransform,
    )>,
    move_and_slide: MoveAndSlide,
) {
    for (entity, ground_detection, collider, collision_layers, global_transform) in &mut query {
        let Some(collider) = ground_detection.cast_shape.as_ref().or(collider) else {
            continue;
        };

        let (_, rotation, translation) = global_transform.to_scale_rotation_translation();

        let up = rotation * Dir3::UP;

        let translation = translation.adjust_precision();
        let rotation = rotation.adjust_precision();

        // Cast the shape downward to check for ground
        let hit = move_and_slide.spatial_query.cast_shape_predicate(
            collider,
            translation,
            rotation,
            -up,
            &ShapeCastConfig::from_max_distance(ground_detection.max_distance),
            &SpatialQueryFilter::from_collision_layers(*collision_layers)
                .with_excluded_entities([entity]),
            // Make sure we don't hit sensors.
            // TODO: Replace this when spatial queries support excluding sensors directly.
            &|entity| move_and_slide.colliders.contains(entity),
        );

        // The character is grounded if we hit a surface that isn't too steep
        let is_grounded = hit.is_some_and(|hit| {
            hit.normal1.angle_between(up.as_vec3()) <= ground_detection.max_angle
        });

        // Update grounded state
        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Performs move-and-slide for character controllers, moving them according to their velocity
/// and sliding along any contact surfaces. Also updates the [`Grounded`] state.
///
/// For simplicity, we assume that the character is not a child entity,
/// and its collider is on the same entity as the `CharacterController`.
fn move_and_slide(
    mut query: Query<
        (
            Entity,
            Option<&GroundDetection>,
            Option<&mut CharacterCollisions>,
            &mut Transform,
            &mut LinearVelocity,
            &Collider,
            &CollisionLayers,
            &TimeFactor,
        ),
        With<CharacterController>,
    >,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
) {
    for (
        entity,
        ground_detection,
        mut collisions,
        mut transform,
        mut lin_vel,
        collider,
        collision_layers,
        time_factor,
    ) in &mut query
    {
        let mut hit_ground_or_ceiling = false;

        if let Some(collisions) = &mut collisions {
            // Clear previous collisions
            collisions.0.clear();
        }

        let up = transform.rotation * Vec3::UP;

        let move_and_slide_config = MoveAndSlideConfig {
            penetration_rejection_threshold: f32::INFINITY,
            ..Default::default()
        };

        // Perform move-and-slide
        let MoveAndSlideOutput {
            position: new_position,
            projected_velocity,
        } = move_and_slide.move_and_slide(
            collider,
            transform.translation.adjust_precision(),
            transform.rotation.adjust_precision(),
            lin_vel.0,
            time.delta().mul_f32(time_factor.0),
            &move_and_slide_config,
            &SpatialQueryFilter::from_collision_layers(*collision_layers)
                .with_excluded_entities([entity]),
            |hit| {
                // This callback is called for each surface we collide with during move-and-slide.
                // In this example, we use it to customize collision behavior for ground surfaces,
                // preventing sliding down slopes when we are grounded, and preventing climbing up steep slopes.

                let Some(ground_detection) = ground_detection else {
                    // Early out if we don't have ground detection.
                    return MoveAndSlideHitResponse::Accept;
                };

                // Determine if the surface is ground based on the angle between the up-vector and the hit normal.
                let angle = up.angle_between(hit.normal.adjust_precision());
                let is_ground = angle <= ground_detection.max_angle;
                let is_ceiling = is_ground && up.dot(hit.normal.adjust_precision()) < 0.0;

                // Decompose the original input velocity into components relative to the hit normal and the up direction,
                // to determine how much of the velocity is contributing to climbing, slipping, and unconstrained movement.
                let [horizontal_component, vertical_component] =
                    split_into_components(lin_vel.0, up);

                // Decompose the horizontal component and the current sliding velocity to determine
                // whether the character is trying to climb or slip, and whether it is actually climbing or slipping.
                let horizontal_velocity_decomposition =
                    decompose_hit_velocity(horizontal_component, *hit.normal, up);
                let decomposition = decompose_hit_velocity(*hit.velocity, *hit.normal, up);

                // An object is trying to slip if the tangential movement induced by its vertical movement
                // points downward (with a small threshold).
                let slipping_intent =
                    up.dot(horizontal_velocity_decomposition.vertical_tangent) < -0.001;

                // An object is slipping if its vertical movement points downward (with a small threshold).
                let slipping = up.dot(decomposition.vertical_tangent) < -0.001;

                // An object is trying to climb if its vertical input motion points upward.
                let climbing_intent = up.dot(vertical_component) > 0.0;

                // An object is climbing if the tangential movement induced by its vertical movement points upward.
                let climbing = up.dot(decomposition.vertical_tangent) > 0.0;

                let projected_velocity = if !is_ground && climbing && !climbing_intent {
                    // Can’t climb the slope, remove the vertical tangent motion induced by the forward motion.
                    decomposition.horizontal_tangent + decomposition.normal_part
                } else if is_ground && slipping && !slipping_intent {
                    // Prevent the vertical movement from sliding down.
                    decomposition.horizontal_tangent + decomposition.normal_part
                } else {
                    // Otherwise, allow full movement (including climbing and slipping)
                    decomposition.horizontal_tangent
                        + decomposition.vertical_tangent
                        + decomposition.normal_part
                };

                // Update the current velocity used by the algorithm.
                *hit.velocity = projected_velocity;

                if is_ground || is_ceiling {
                    // We hit a ground or ceiling surface!
                    hit_ground_or_ceiling = true;
                }

                if let Some(collisions) = &mut collisions {
                    // Record the collision for use in other systems, such as applying forces to dynamic bodies.
                    collisions.0.push(CharacterCollision {
                        collider: hit.entity,
                        point: hit.point,
                        normal: *hit.normal,
                        character_velocity: *hit.velocity,
                    });
                }

                // Accept the hit and continue the move-and-slide algorithm with the modified velocity.
                MoveAndSlideHitResponse::Accept
            },
        );

        // Update position to the final position calculated by move-and-slide.
        transform.translation = new_position.f32();

        // If we hit the ground or a ceiling, update the velocity along the up-direction
        // to prevent accumulating velocity along the ground normal when hitting slopes,
        // and to prevent sticking to ceilings when jumping.
        // if hit_ground_or_ceiling {
        //     let up = up.adjust_precision();
        //     let velocity_along_up = lin_vel.dot(up);
        //     let new_velocity_along_up = projected_velocity.dot(up);
        //     lin_vel.0 += (new_velocity_along_up - velocity_along_up) * up;
        // }

        lin_vel.0 = projected_velocity;
    }
}

/// The decomposition of a velocity vector into parts relative to a collision normal and an up-direction.
///
/// This is used for determining how much of the velocity is contributing to climbing, slipping, and unconstrained movement.
#[derive(Debug)]
struct VelocityDecomposition {
    /// The part of the velocity that is directly against the collision normal.
    normal_part: Vector,
    /// The part of the velocity that is tangent to the collision surface and perpendicular to the up-direction.
    horizontal_tangent: Vector,
    /// The part of the velocity that is tangent to the collision surface and parallel to the up-direction.
    vertical_tangent: Vector,
}

/// Decomposes a velocity vector into parts relative to a collision `normal` and an `up` direction.
fn decompose_hit_velocity(velocity: Vector, normal: Dir, up: Vector) -> VelocityDecomposition {
    let normal = normal.adjust_precision();
    let normal_part = normal * normal.dot(velocity);
    let tangent_part = velocity - normal_part;

    let horizontal_tangent_dir = normal.cross(up).normalize_or_zero();
    let horizontal_tangent = tangent_part.dot(horizontal_tangent_dir) * horizontal_tangent_dir;
    let vertical_tangent = tangent_part - horizontal_tangent;

    VelocityDecomposition {
        normal_part,
        horizontal_tangent,
        vertical_tangent,
    }
}

/// Splits a vector into horizontal and vertical components relative to a given `up` direction.
fn split_into_components(v: Vector, up: Vector) -> [Vector; 2] {
    let vertical_component = up * v.dot(up);
    let horizontal_component = v - vertical_component;
    [horizontal_component, vertical_component]
}
