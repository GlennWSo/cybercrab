use std::f32::consts::PI;

use bevy::{
    input::{common_conditions::input_just_released, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
};

const PI2: f32 = PI / 2.0;
const PLAYER_SPEED: f32 = 2000.0;

fn spawn_camera(mut cmd: Commands) {
    cmd.spawn((Camera3d::default(), Player));
}

fn spawn_map(
    mut cmd: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    let ball_mesh = mesh_assets.add(Sphere::new(1.0));
    for h in 0..16 {
        let color = Color::hsl((h as f32 / 16.0) * 360.0, 1.0, 0.5);
        let ball_material = material_assets.add(StandardMaterial {
            base_color: color,
            ..Default::default()
        });
        let x = -8.0 + h as f32;

        let transform = Transform::from_translation(Vec3::new(x, 0.0, -50.0));
        cmd.spawn((
            transform,
            Mesh3d(ball_mesh.clone()),
            MeshMaterial3d(ball_material),
        ));
    }
}

fn player_look(
    mut player: Single<&mut Transform, With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
) {
    if !window.focused {
        return;
    }
    let dt = time.delta_secs();
    let sensitivity = 50.0 / window.width().min(window.height());
    use EulerRot::YXZ;
    let (mut yaw, mut pitch, _) = player.rotation.to_euler(YXZ);
    yaw -= mouse_motion.delta.x * dt * sensitivity;
    pitch -= mouse_motion.delta.y * dt * sensitivity;
    pitch = pitch.clamp(-PI2, PI2);
    player.rotation = Quat::from_euler(YXZ, yaw, pitch, 0.0);
}

fn player_move(
    mut player: Single<&mut Transform, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let mut delta = Vec3::ZERO;
    if input.pressed(KeyCode::KeyA) {
        delta.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        delta.x += 1.0;
    }
    if input.pressed(KeyCode::KeyW) {
        delta.z += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        delta.z -= 1.0;
    }
    let forward = player.forward().as_vec3() * delta.z;
    let right = player.right().as_vec3() * delta.x;
    let mut to_move = forward + right;
    to_move.y = 0.0;
    to_move = to_move.normalize_or_zero() * dt;
    println!("{:#?}", to_move);
    player.translation += to_move * dt * PLAYER_SPEED;
}

fn apply_grab(grab: Trigger<GrabEvent>, mut window: Single<&mut Window, With<PrimaryWindow>>) {
    if **grab {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    } else {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn focus_events(mut events: EventReader<WindowFocused>, mut cmd: Commands) {
    if let Some(event) = events.read().last() {
        cmd.trigger(GrabEvent(event.focused));
    }
}

fn toggle_grab(mut window: Single<&mut Window, With<PrimaryWindow>>, mut cmd: Commands) {
    window.focused = !window.focused;
    cmd.trigger(GrabEvent(window.focused));
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (spawn_camera, spawn_map));
    app.add_systems(Update, (player_look, player_move).chain());
    app.add_systems(
        Update,
        (
            player_look,
            focus_events,
            toggle_grab.run_if(input_just_released(KeyCode::Escape)),
        ),
    );
    app.add_observer(apply_grab);
    app.run();
}

#[derive(Component)]
struct Player;

#[derive(Event, Deref)]
struct GrabEvent(bool);
