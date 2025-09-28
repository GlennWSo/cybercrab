use std::f32::consts::PI;

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

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

const PI2: f32 = PI / 2.0;

fn player_look(
    mut player: Single<&mut Transform, With<Player>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let sensitivity = 0.1;
    use EulerRot::YXZ;
    let (mut yaw, mut pitch, _) = player.rotation.to_euler(YXZ);
    yaw -= mouse_motion.delta.x * dt * sensitivity;
    pitch -= mouse_motion.delta.y * dt * sensitivity;
    pitch = pitch.clamp(-PI2, PI2);
    player.rotation = Quat::from_euler(YXZ, yaw, pitch, 0.0);
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (spawn_camera, spawn_map));
    app.add_systems(Update, player_look);
    app.run();
}

#[derive(Component)]
struct Player;
