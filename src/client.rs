use std::sync::{Arc, Mutex};

use ambient_api::{
    core::{
        hierarchy::components::parent, messages::Frame, player::components::user_id,
        rendering::components::color,
    },
    element::{use_entity_component, use_query},
    prelude::*,
};
use packages::this::{
    components::{balls_per_frame, input_timestamp, latency, local_latency},
    messages::{ChangeBallRate, Input, ReportLatency, SpawnBalls},
};

const FRAMES_PER_INPUT: usize = 10;
const FRAMES_PER_LATENCY_REPORT: usize = 60;
const LATENCY_SMOOTHING: u32 = 4;

#[main]
pub fn main() {
    let current_rate = Arc::new(Mutex::new(0));
    query(balls_per_frame()).each_frame({
        let current_rate = current_rate.clone();
        move |results| {
            if let Some((_, rate)) = results.first() {
                *current_rate.lock().unwrap() = *rate;
            }
        }
    });

    entity::add_component(entity::resources(), local_latency(), Duration::ZERO);
    let player_id = player::get_local();
    let mut frame_no = 0;
    Frame::subscribe(move |_| {
        let (input, _) = input::get_delta();
        if input.keys.contains(&KeyCode::Q) || input.keys.contains(&KeyCode::E) {
            let mut rate = *current_rate.lock().unwrap();
            if input.keys.contains(&KeyCode::Q) {
                rate = rate.saturating_sub(1);
            }
            if input.keys.contains(&KeyCode::E) {
                rate = rate.saturating_add(1);
            }
            ChangeBallRate { rate }.send_server_reliable();
        }

        if input.keys.contains(&KeyCode::R) {
            SpawnBalls { count: 100 }.send_server_reliable();
        }

        if frame_no % FRAMES_PER_INPUT == 0 {
            Input {
                timestamp: game_time(),
            }
            .send_server_unreliable();
        }

        if frame_no % FRAMES_PER_LATENCY_REPORT == 0 {
            ReportLatency {
                latency: entity::get_component(entity::resources(), local_latency()).unwrap(),
            }
            .send_server_reliable();
        }

        frame_no += 1;
    });

    query((parent(), input_timestamp())).each_frame(move |entities| {
        let now = game_time();
        for (id, (parent_id, timestamp)) in entities {
            if parent_id != player_id {
                continue;
            }
            let current_latency = now - timestamp;
            entity::mutate_component(entity::resources(), local_latency(), |latency| {
                *latency =
                    (*latency * (LATENCY_SMOOTHING - 1) + current_latency) / LATENCY_SMOOTHING;
            });
            entity::despawn(id);
        }
    });

    Status.el().spawn_interactive();
}

#[element_component]
fn Status(hooks: &mut Hooks) -> Element {
    let mut elements = Vec::new();
    let rate = use_entity_component(hooks, entity::synchronized_resources(), balls_per_frame())
        .unwrap_or_default();
    elements.push(Text::el(format!("Balls per frame: {}", rate)));
    let input_latency = use_entity_component(hooks, entity::resources(), local_latency()).unwrap();
    elements.push(Text::el(format!("Input latency: {:?}", input_latency)));
    let mut player_latencies = use_query(hooks, (user_id(), latency()));
    player_latencies.sort_unstable();
    let local_player_id = player::get_local();
    elements.extend(
        player_latencies
            .into_iter()
            .map(|(id, (user_id, latency))| {
                let el = Text::el(format!("{} latency: {:?}", user_id, latency));
                if id == local_player_id {
                    el.with(color(), vec4(1., 1., 1., 1.))
                } else {
                    el
                }
            }),
    );
    FlowColumn::el(elements)
}
