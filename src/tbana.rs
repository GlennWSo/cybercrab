use std::borrow::Cow;

use avian3d::prelude::{Collider, CollidingEntities, CollisionLayers, LinearVelocity};
use bevy::color::palettes::css;
use bevy::log::tracing::Instrument;
use bevy::prelude::{Mesh3d, *};

use crate::fotocell::{
    on_fotocell_blocked, on_fotocell_unblocked, Fotocell, FotocellAssets, FotocellBundle,
};
use crate::io::{Dio, DioPin, IoDevices, NodeId, Switch};
use crate::physics::PhysLayer;
use crate::shiftreg::RegisterPosition;
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
        app.add_systems(Update, (motor_effect, tbana_motor_logic, tbana_ready));
        app.add_observer(on_insert_tbana);
    }
}

fn tbana_motor_logic(
    banor: Query<(&Children, &Direction), With<TransportBana>>,
    sensors: Query<(&DioPin, &NodeId), (With<Fotocell>, Without<TransportBana>, Without<Movimot>)>,
    motors: Query<&Movimot, (Without<TransportBana>, Without<Fotocell>)>,
    mut io: ResMut<IoDevices>,
) {
    for (children, direction) in banor {
        let sensors = children.iter().filter_map(|id| sensors.get(id).ok());
        let num_on_fotocells = sensors
            .filter_map(|(pin, node)| io.get_input_bit(*node, *pin))
            .filter(|v| *v)
            .count();
        let run = num_on_fotocells > 1;

        let motors = children.iter().filter_map(|id| motors.get(id).ok());
        for motor in motors {
            io.set_output_bit(motor.dq.forward.address, motor.dq.forward.pin, false);
            io.set_output_bit(motor.dq.reverse.address, motor.dq.reverse.pin, false);
            let (address, pin) = match direction {
                Direction::Forward => (motor.dq.forward.address, motor.dq.forward.pin),
                Direction::Reverse => (motor.dq.reverse.address, motor.dq.reverse.pin),
            };
            io.set_output_bit(address, pin, run);
        }
    }
}

#[derive(Component, Reflect, Debug, Copy, Clone)]
pub enum SensorRole {
    FrontEnd,
    FrontProximity,
    BackEnd,
    BackProximity,
}

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
pub enum Direction {
    #[default]
    Forward,
    Reverse,
}

#[derive(Event, Clone)]
pub struct InsertTbana4x2 {
    entity: Entity,
    parrent: Option<Entity>,
    name: Cow<'static, str>,
    io_inputs: [Dio; 4],
    io_outputs: [Dio; 2 * 3],
    transform: Transform,
    direction: Direction,
    register_pos: RegisterPosition,
    push_to: Option<PushTo>,
    pull_from: Option<PullFrom>,
}

impl InsertTbana4x2 {
    pub fn new(
        entity: Entity,
        parrent: Option<Entity>,
        name: impl Into<Cow<'static, str>>,
        io_inputs: [Dio; 4],
        io_outputs: [Dio; 2 * 3],
        transform: Transform,
        direction: Direction,
        register_pos: RegisterPosition,
        push_to: Option<PushTo>,
        pull_from: Option<PullFrom>,
    ) -> Self {
        Self {
            entity,
            parrent,
            name: name.into(),
            io_inputs,
            io_outputs,
            transform,
            direction,
            register_pos,
            push_to,
            pull_from,
        }
    }
}

fn tbana_ready(
    tbanor: Query<(&mut Readiness, &Children), With<TransportBana>>,
    sensors: Query<(&NodeId, &DioPin), (With<Switch>, Without<TransportBana>)>,
    io: Res<IoDevices>,
) {
    for (mut tbana, children) in tbanor {
        let sensors = children
            .iter()
            .filter_map(|entity| sensors.get(entity).ok());
        let count = sensors
            .filter(|(node, pin)| io.get_input_bit(**node, **pin).unwrap())
            .count();
        let ready = match count {
            0 => Readiness::Ready { has_detail: false },
            4 => Readiness::Ready { has_detail: true },
            1..=3 => Readiness::Busy,
            _ => todo!(),
        };
        *tbana = ready;
    }
}

fn motor_effect(
    target: Query<(&CollidingEntities, &mut LinearVelocity), Without<Movimot>>,
    motors: Query<(&Movimot, &Transform)>,
    io: Res<IoDevices>,
) {
    for (colliding, mut velocity) in target {
        let motors = colliding.iter().filter_map(|id| motors.get(*id).ok());
        let speeds: Vec<_> = motors
            .map(|(motor, transform)| {
                let fw_node = motor.dq.forward.address;
                let fw_pin = motor.dq.forward.pin;
                let rev_node = motor.dq.reverse.address;
                let rev_pin = motor.dq.reverse.pin;
                let fw = io.get_output_bit(fw_node, fw_pin);
                let rev = io.get_output_bit(motor.dq.reverse.address, motor.dq.reverse.pin);
                let rapid =
                    io.get_output_bit(motor.dq.rapid.address, motor.dq.rapid.pin) == Some(true);
                let speed = if rapid {
                    motor.fast_speed
                } else {
                    motor.slow_speed
                };
                match (fw, rev) {
                    (Some(true), Some(true)) => {
                        eprintln!("motor cant run in both directions");
                        eprintln!("motor FW signal from address{:?}, pin{:?}", fw_node, fw_pin);
                        eprintln!(
                            "motor rev signal from address{:?}, pin{:?}",
                            rev_node, rev_pin
                        );
                        dbg!(Vec3::ZERO)
                    }
                    (Some(true), _) => speed * transform.left(),
                    (_, Some(true)) => -speed * transform.left(),
                    // (None, None) => 0_f32,
                    _ => Vec3::ZERO,
                }
            })
            .collect();
        let n = speeds.len();
        if n == 0 {
            velocity.0 = Vec3::ZERO;
            continue;
        }
        let avg_speed = speeds.into_iter().sum::<Vec3>() / (n as f32);
        velocity.0 = avg_speed;
    }
}

// fn tbana_motor_effects(motors: Query<(&CollidingEntities, &MovimotDQ)>, io: Res<IoDevices>) {}

fn on_insert_tbana(
    spawn: On<InsertTbana4x2>,
    mut cmd: Commands,
    fotocell_assets: Res<FotocellAssets>,
    tbana_assets: Res<TBanaAssets>,
) {
    let z_values = [-0.9, -0.7, 0.7, 0.9];
    let fc_names = ["forward_end", "forward_slow", "reverse_slow", "reverse_end"];
    let io_inputs = spawn.io_inputs.iter();
    let phys_layers = CollisionLayers::new(PhysLayer::Sensor, PhysLayer::Detail);
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
            cmd.spawn((fotocell, transform, phys_layers))
                .observe(on_fotocell_blocked)
                .observe(on_fotocell_unblocked)
                .id()
        })
        .collect();

    let z_values = [-0.8, 0.8];
    let mut io_outputs = spawn.io_outputs.iter();
    let phys_layers = CollisionLayers::new(PhysLayer::Actuator, PhysLayer::Detail);
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
            cmd.spawn((bundle, transform, phys_layers)).id()
        })
        .collect();

    let bana_bundle = (
        TbanaBundle::new(&tbana_assets),
        spawn.transform,
        Name::new(spawn.name.clone()),
        spawn.direction,
        spawn.register_pos,
    );
    match spawn.parrent {
        Some(parrent) => {
            cmd.entity(spawn.entity)
                .insert((bana_bundle, ChildOf(parrent)))
                .add_children(&fotocells[0..4])
                .add_children(&motors_wheels[0..2]);
        }
        None => {
            cmd.entity(spawn.entity)
                .insert(bana_bundle)
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
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub mode: Mode,
    pub ready: Readiness,
}

impl TbanaBundle {
    pub fn new(tbana_assets: &TBanaAssets) -> Self {
        Self {
            tbana: TransportBana,
            auto: AutoMode::default(),
            mesh: Mesh3d(tbana_assets.bana_mesh.clone()),
            material: MeshMaterial3d(tbana_assets.bana_materials.ready.clone()),
            mode: default(),
            ready: default(),
        }
    }
}

#[derive(Component, Reflect)]
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

#[derive(Component, Reflect, Copy, Clone, Debug)]
pub struct MovimotDQ {
    forward: Dio,
    reverse: Dio,
    rapid: Dio,
}

impl MovimotDQ {
    #[allow(dead_code)]
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

#[derive(Component, Reflect, Debug)]
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

#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub enum Readiness {
    #[default]
    Busy,
    Disabled,
    Ready {
        has_detail: bool,
    },
}

#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub enum Mode {
    #[default]
    Push,
    Pull,
}

#[derive(Component, Reflect, Clone, Copy)]
#[relationship(relationship_target = Reciver )]
/// Pushes production details to other
pub struct PushTo(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship=PushTo)]
pub struct Reciver(Vec<Entity>);

#[derive(Component, Reflect, Clone, Copy)]
#[relationship(relationship_target = Giver )]
pub struct PullFrom(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship=PullFrom)]
pub struct Giver(Vec<Entity>);
