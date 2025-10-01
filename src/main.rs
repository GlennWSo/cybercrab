use std::f32::consts::PI;

use avian3d::prelude::{Collider, CollidingEntities, RigidBody};
use bevy::{
    color::palettes::css,
    input::{common_conditions::input_just_released, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy_polyline::prelude::{
    Polyline, PolylineBundle, PolylineHandle, PolylineMaterial, PolylineMaterialHandle,
};
use cybercrab::{DummyPlugin, InitSet};

const PI2: f32 = PI / 2.0;
const PLAYER_SPEED: f32 = 1000.0;

fn spawn_camera(mut cmd: Commands) {
    let mut position = Vec3::default();
    position.z = 50.0;
    cmd.spawn((
        Camera3d::default(),
        Player,
        Transform::from_translation(position),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Msaa::Sample4,
    ));
}

fn spawn_map(
    mut cmd: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    let ball_mesh = mesh_assets.add(Sphere::new(1.0));
    for h in 0..6 {
        let color = Color::hsl((h as f32 / 6.0) * 360.0, 1.0, 0.5);
        let ball_material = material_assets.add(StandardMaterial {
            base_color: color,
            ..Default::default()
        });
        let x = -8.0 + h as f32;

        let transform = Transform::from_translation(Vec3::new(x, 10.0, 0.0));
        cmd.spawn((
            Name::new("DummyBall"),
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
    window: Single<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    if !window.focused {
        return;
    }
    let dt = time.delta_secs();
    let mut delta = Vec3::ZERO;
    let mut to_move = Vec3::ZERO;

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
    if input.pressed(KeyCode::Space) {
        to_move.y += 0.5;
    }
    if input.pressed(KeyCode::KeyC) {
        to_move.y -= 0.5;
    }

    let mut xz_direction = player.forward().as_vec3();
    xz_direction.y = 0.0;
    xz_direction = xz_direction.normalize();
    let forward = xz_direction * delta.z;
    let to_right = Quat::from_rotation_y(-90_f32.to_radians());
    let right = (to_right * xz_direction) * delta.x;
    to_move += forward + right;
    to_move = to_move.normalize_or_zero() * dt;
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

fn handle_focus_events(mut events: EventReader<WindowFocused>, mut cmd: Commands) {
    if let Some(event) = events.read().last() {
        cmd.trigger(GrabEvent(event.focused));
    }
}

fn toggle_grab(mut window: Single<&mut Window, With<PrimaryWindow>>, mut cmd: Commands) {
    window.focused = !window.focused;
    cmd.trigger(GrabEvent(window.focused));
}

fn shoot_ball(
    input: Res<ButtonInput<MouseButton>>,
    player: Single<&Transform, With<Player>>,
    mut spawner: EventWriter<BallSpawn>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if window.cursor_options.visible {
        return;
    }
    if !input.just_pressed(MouseButton::Left) {
        return;
    }
    spawner.write(BallSpawn {
        position: player.translation,
    });
}

fn spawn_ball(
    mut events: EventReader<BallSpawn>,
    mut cmd: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
) {
    for spawn in events.read() {
        let mat_handle = mat_assets.add(StandardMaterial::default());
        let mesh_handle = mesh_assets.add(Sphere::new(1.0));
        let bundle = (
            Transform::from_translation(spawn.position),
            MeshMaterial3d(mat_handle),
            Mesh3d(mesh_handle),
            RigidBody::Kinematic,
            Collider::sphere(1.0),
            CollidingEntities::default(),
        );
        cmd.spawn(bundle);
    }
}
fn debug_collide(query: Query<(Entity, &CollidingEntities)>) {
    for (entity, colliding_entities) in &query {
        println!(
            "{} is colliding with the following entities: {:?}",
            entity, colliding_entities,
        );
    }
}
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DummyPlugin);
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(WorldInspectorPlugin::new());

    app.add_systems(Startup, (spawn_camera, spawn_map).in_set(InitSet::Spawn));
    app.add_systems(Update, (player_look, player_move, debug_collide).chain());
    app.add_systems(
        Update,
        (
            player_look,
            handle_focus_events,
            toggle_grab.run_if(input_just_released(KeyCode::Escape)),
        ),
    );
    app.add_systems(
        Update,
        (
            spawn_ball,
            shoot_ball.before(spawn_ball).before(handle_focus_events),
        ),
    );
    app.add_observer(apply_grab);
    app.add_event::<BallSpawn>();
    app.run();
}

#[derive(Component)]
struct Player;

#[derive(Event, Deref)]
struct GrabEvent(bool);

#[derive(Event)]
struct BallSpawn {
    position: Vec3,
}
