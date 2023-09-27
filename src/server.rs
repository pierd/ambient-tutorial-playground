use ambient_api::{
    core::{
        hierarchy::components::parent,
        messages::Frame,
        model::components::model_from_url,
        physics::components::{cube_collider, dynamic, plane_collider, sphere_collider},
        player::components::is_player,
        primitives::{
            components::{cube, quad},
            concepts::Sphere,
        },
        rendering::components::color,
        transform::{
            components::{scale, translation},
            concepts::{Transformable, TransformableOptional},
        },
    },
    entity::despawn,
    prelude::*,
    rand,
};
use packages::{
    base_assets,
    character_animation::components::basic_character_animations,
    fps_controller::components::use_fps_controller,
    this::{
        assets,
        components::{balls_per_frame, balls_to_spawn, bouncy_created, input_timestamp, latency},
        messages::{ChangeBallRate, Input, ReportLatency, SpawnBalls},
    },
};

fn spawn_ball(with_ttl: bool) {
    let mut ball = Entity::new()
        .with_merge(Sphere::suggested())
        .with_merge(Transformable::suggested())
        .with(scale(), Vec3::ONE * 0.2)
        .with(
            translation(),
            Vec3::X * 10. + (rand::random::<Vec2>() * 2.0 - 1.0).extend(10.),
        )
        .with(sphere_collider(), 0.5)
        .with(dynamic(), true);
    if with_ttl {
        ball = ball.with(bouncy_created(), game_time())
    }
    ball.spawn();
}

#[main]
pub fn main() {
    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.0)
        .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
        .with(plane_collider(), ())
        .spawn();

    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with(use_fps_controller(), ())
                    .with(model_from_url(), base_assets::assets::url("Y Bot.fbx"))
                    .with(basic_character_animations(), id)
                    .with(latency(), Duration::MAX),
            );
        }
    });

    for _ in 0..30 {
        Entity::new()
            .with(cube(), ())
            .with(cube_collider(), Vec3::ONE * 0.5)
            .with(
                translation(),
                (rand::random::<Vec2>() * 20.0 - 10.0).extend(1.),
            )
            .spawn();
    }

    entity::add_component(entity::synchronized_resources(), balls_per_frame(), 0);
    entity::add_component(entity::resources(), balls_to_spawn(), 0);

    Frame::subscribe(move |_| {
        let rate =
            entity::get_component(entity::synchronized_resources(), balls_per_frame()).unwrap();
        for _ in 0..rate {
            spawn_ball(true);
        }

        if entity::get_component(entity::resources(), balls_to_spawn()).unwrap_or_default() > 0 {
            entity::mutate_component(entity::resources(), balls_to_spawn(), |c| *c -= 1);
            spawn_ball(true);
        }
    });

    query(bouncy_created()).each_frame(|entities| {
        for (id, created) in entities {
            if (game_time() - created).as_secs_f32() > 5.0 {
                despawn(id);
            }
        }
    });

    ChangeBallRate::subscribe(move |_, msg| {
        entity::set_component(
            entity::synchronized_resources(),
            balls_per_frame(),
            msg.rate,
        );
    });

    Input::subscribe(|ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };
        Entity::new()
            .with(parent(), player_id)
            .with(input_timestamp(), msg.timestamp)
            .spawn();
    });

    ReportLatency::subscribe(|ctx, msg| {
        let Some(player_id) = ctx.client_entity_id() else {
            return;
        };
        entity::set_component(player_id, latency(), msg.latency);
    });

    SpawnBalls::subscribe(|_, msg| {
        entity::mutate_component(entity::resources(), balls_to_spawn(), |c| *c += msg.count);
    });

    // Entity::new()
    //     .with_merge(Transformable {
    //         local_to_world: Default::default(),
    //         optional: TransformableOptional {
    //             scale: Some(Vec3::ONE * 0.3),
    //             ..Default::default()
    //         },
    //     })
    //     .with(model_from_url(), assets::url("AntiqueCamera.glb"))
    //     .spawn();
}
