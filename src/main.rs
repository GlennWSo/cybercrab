use bevy::prelude::*;

fn spawn_camera(mut cmd: Commands) {
    cmd.spawn(Camera3d::default());
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

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (spawn_camera, spawn_map));
    app.run();
}
