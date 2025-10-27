use std::borrow::Cow;

use avian3d::prelude::Collider;
use bevy::color::palettes::css;
use bevy::prelude::{Mesh3d, *};
use itertools::Itertools;

use crate::fotocell::{on_fotocell_blocked, on_fotocell_unblocked, FotocellAssets, FotocellBundle};
use crate::io::{Dio, DioPin, IoDevices, NodeId};
use crate::shiftreg::Slot;
use crate::sysorder::InitSet;

pub struct TbanaPlugin;

impl Plugin for TbanaPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AutoMode>();
        app.register_type::<PushTo>();
        app.register_type::<Reciver>();
        app.register_type::<PullFrom>();
        app.register_type::<Giver>();
        app.register_type::<Movimot>();
        app.init_resource::<TBanaAssets>();
        app.add_systems(Startup, load_assets.in_set(InitSet::LoadAssets));
        app.add_observer(on_spawn_tbana);
    }
}

#[derive(Event, Clone)]
pub struct SpawnTbana4x2 {
    parrent: Option<Entity>,
    name: Cow<'static, str>,
    io_inputs: [Dio; 4],
    io_outputs: [Dio; 2 * 3],
    transform: Transform,
}

impl SpawnTbana4x2 {
    pub fn new(
        parrent: Option<Entity>,
        name: impl Into<Cow<'static, str>>,
        io_inputs: [Dio; 4],
        io_outputs: [Dio; 2 * 3],
        transform: Transform,
    ) -> Self {
        Self {
            parrent,
            name: name.into(),
            io_inputs,
            io_outputs,
            transform,
        }
    }
}

fn on_spawn_tbana(
    spawn: On<SpawnTbana4x2>,
    mut cmd: Commands,
    fotocell_assets: Res<FotocellAssets>,
    tbana_assets: Res<TBanaAssets>,
) {
    let z_values = [-0.9, -0.7, 0.7, 0.9];
    let fc_names = ["forward_end", "forward_slow", "reverse_slow", "reverse_end"];
    let io_inputs = spawn.io_inputs.iter();
    let fotocells: Vec<_> = io_inputs
        .zip(z_values)
        .zip(fc_names)
        .map(|((dio, z), name)| {
            let coord = Vec3 {
                x: 0.45,
                y: 0.53,
                z,
            };
            let mut transform = Transform::from_translation(coord);
            transform.rotate_local_y(-90_f32.to_radians());
            let fotocell = FotocellBundle::new(name, dio.pin, &fotocell_assets, dio.address, 0.8);
            cmd.spawn((fotocell, transform))
                .observe(on_fotocell_blocked)
                .observe(on_fotocell_unblocked)
                .id()
        })
        .collect();

    let z_values = [-0.8, 0.8];
    let mut io_outputs = spawn.io_outputs.iter();
    let motors_wheels: Vec<_> = z_values
        .into_iter()
        .map(|z| {
            let forward = *io_outputs.next().unwrap();
            let reverse = *io_outputs.next().unwrap();
            let rapid = *io_outputs.next().unwrap();
            let bundle = TransportWheelBundle::new(
                &tbana_assets,
                MovimotDQ {
                    forward,
                    reverse,
                    rapid,
                },
            );
            let mut transform = Transform::from_xyz(0.0, 0.45, z);
            transform.rotate_local_y(90_f32.to_radians());
            cmd.spawn((bundle, transform)).id()
        })
        .collect();

    let bundle = (
        TbanaBundle::new(&tbana_assets),
        spawn.transform,
        Name::new(spawn.name.clone()),
    );
    match spawn.parrent {
        Some(parrent) => {
            cmd.spawn((bundle, ChildOf(parrent)))
                .add_children(&fotocells[0..4])
                .add_children(&motors_wheels[0..2]);
        }
        None => {
            cmd.spawn(bundle)
                .add_children(&fotocells[0..4])
                .add_children(&motors_wheels[0..2]);
        }
    };
}

pub fn load_assets(
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut tbana_res: ResMut<TBanaAssets>,
) {
    let ready_mode = material_assets.add(StandardMaterial {
        base_color: css::CORNSILK.into(),
        ..Default::default()
    });
    let run_mode = material_assets.add(StandardMaterial {
        base_color: css::CHARTREUSE.into(),
        ..Default::default()
    });
    let alarm_mode = material_assets.add(StandardMaterial {
        base_color: css::CRIMSON.into(),
        ..Default::default()
    });
    tbana_res.bana_materials.ready = ready_mode;
    tbana_res.bana_materials.running = run_mode;
    tbana_res.bana_materials.alarm = alarm_mode;
    let shade = 0.2;
    let ready_mode = material_assets.add(StandardMaterial {
        base_color: css::CORNSILK.darker(shade).into(),
        ..Default::default()
    });
    let run_mode = material_assets.add(StandardMaterial {
        base_color: css::CHARTREUSE.darker(shade).into(),
        ..Default::default()
    });
    let alarm_mode = material_assets.add(StandardMaterial {
        base_color: css::CRIMSON.darker(shade).into(),
        ..Default::default()
    });
    tbana_res.wheel_materials.ready = ready_mode;
    tbana_res.wheel_materials.running = run_mode;
    tbana_res.wheel_materials.alarm = alarm_mode;

    tbana_res.bana_mesh = mesh_assets.add(Extrusion::new(Rectangle::default(), 2.0));
    tbana_res.wheel_mesh = mesh_assets.add(Extrusion::new(Circle::new(0.1), 0.6));
}

#[derive(Default)]
struct ModeMaterials {
    ready: Handle<StandardMaterial>,
    running: Handle<StandardMaterial>,
    alarm: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct TBanaAssets {
    bana_mesh: Handle<Mesh>,
    bana_materials: ModeMaterials,
    wheel_materials: ModeMaterials,
    wheel_mesh: Handle<Mesh>,
    wheel_collider: Collider,
}

#[derive(Bundle)]
pub struct TbanaBundle {
    pub tbana: TransportBana,
    pub auto: AutoMode,
    pub slot: Slot,
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
}

impl TbanaBundle {
    pub fn new(tbana_assets: &TBanaAssets) -> Self {
        Self {
            tbana: TransportBana,
            auto: AutoMode::default(),
            slot: Slot::default(),
            mesh: Mesh3d(tbana_assets.bana_mesh.clone()),
            material: MeshMaterial3d(tbana_assets.bana_materials.ready.clone()),
        }
    }
}

#[derive(Component)]
pub struct TransportBana;

#[derive(Reflect, Component, Default, Deref)]
pub struct AutoMode {
    enabled: bool,
}

#[derive(Component)]
pub struct Wheel;

// enum MoviMotion {
//     FastForward,
//     Forward,
//     FastReverse,
//     Reverse,
//     Stop,
// }

#[derive(Component, Reflect)]
pub struct MovimotDQ {
    forward: Dio,
    reverse: Dio,
    rapid: Dio,
}

impl MovimotDQ {
    pub fn new(address: u32, fw: u16, rev: u16, rapid: u16) -> Self {
        let address = NodeId(address);
        let forward = Dio {
            address,
            pin: DioPin(fw),
        };
        let reverse = Dio {
            address,
            pin: DioPin(rev),
        };

        let rapid = Dio {
            address,
            pin: DioPin(rapid),
        };

        Self {
            forward,
            reverse,
            rapid,
        }
    }
}

#[derive(Component, Reflect)]
pub struct Movimot {
    // pub motion: MoviMotion,
    pub fast_speed: f32,
    pub slow_speed: f32,
    pub dq: MovimotDQ,
}

impl Movimot {}

#[derive(Bundle)]
pub struct TransportWheelBundle {
    marker: Wheel,
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    collider: Collider,
    motor: Movimot,
}

impl TransportWheelBundle {
    pub fn new(assets: &TBanaAssets, dq: MovimotDQ) -> Self {
        Self {
            marker: Wheel,
            mesh: Mesh3d(assets.wheel_mesh.clone()),
            material: MeshMaterial3d(assets.wheel_materials.ready.clone()),
            collider: assets.wheel_collider.clone(),
            motor: Movimot {
                dq,
                fast_speed: 10.0,
                slow_speed: 2.0,
            },
        }
    }
}

#[derive(Component, Reflect)]
#[relationship(relationship_target = Reciver )]
/// Pushes production details to other
pub struct PushTo(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship=PushTo)]
pub struct Reciver(Vec<Entity>);

#[derive(Component, Reflect)]
#[relationship(relationship_target = Giver )]
pub struct PullFrom(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship=PullFrom)]
pub struct Giver(Vec<Entity>);
