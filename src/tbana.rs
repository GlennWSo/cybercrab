use std::borrow::Cow;

use avian3d::prelude::{Collider, CollidingEntities, CollisionLayers, LinearVelocity};
use bevy::color::palettes::css;
use bevy::prelude::{Mesh3d, *};

use crate::fotocell::{
    on_fotocell_blocked, on_fotocell_unblocked, Fotocell, FotocellAssets, FotocellBundle,
};
use crate::io::{Dio, DioPin, IoDevices, NodeId, Switch};
use crate::physics::PhysLayer;
use crate::sensor::{PositionReached, SensorPosition};
use crate::shiftreg::{Register, RegisterPosition, ShiftOver};
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
        app.add_message::<PushRequest>();
        app.init_resource::<TBanaAssets>();
        app.add_systems(Startup, load_assets.in_set(InitSet::LoadAssets));
        app.add_systems(
            Update,
            (
                motor_effect,
                // tbana_motor_logic,
                push_request_handler,
                request_push,
                stop_pushing,
                set_tbana_ready,
            ),
        );
        app.add_observer(on_insert_tbana);
        app.add_observer(on_switch_tbana_direction);
    }
}

#[derive(Event)]
pub struct SwitchDirection;

fn on_switch_tbana_direction(
    trigger: On<SwitchDirection>,
    mut cmd: Commands,
    mut tbanor: Query<(
        Entity,
        &mut Direction,
        &mut TransportState,
        &PushTo,
        &RegisterPosition,
    )>,
    reg: Res<Register>,
) {
    for (me, mut dir, mut state, pushto, pos) in tbanor.iter_mut() {
        match *dir.as_ref() {
            Direction::Forward => *dir = Direction::Reverse,
            Direction::Reverse => *dir = Direction::Forward,
        }
        // cmd.entity(ent).clear_related::<PushTo>();
        let push_to = pushto.0;
        cmd.entity(push_to).remove_related::<PushTo>(&[me]);
        cmd.entity(me).add_related::<PushTo>(&[push_to]);
        // cmd.entity(from).
        match *state.as_ref() {
            TransportState::Reciving => {
                *state = TransportState::ReadySend;
            }
            TransportState::Sending => {
                cmd.trigger(ShiftOver {
                    from: me,
                    to: push_to,
                });
                cmd.trigger(StartRecive(me));
                cmd.trigger(StartSending { entity: push_to });
            }
            _ => (),
        }
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
            io.set_output_bit(motor.dq.forward.node, motor.dq.forward.pin, false);
            io.set_output_bit(motor.dq.reverse.node, motor.dq.reverse.pin, false);
            let (address, pin) = match direction {
                Direction::Forward => (motor.dq.forward.node, motor.dq.forward.pin),
                Direction::Reverse => (motor.dq.reverse.node, motor.dq.reverse.pin),
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

fn stop_pushing(
    mut cmd: Commands,
    pushers: Query<(Entity, &PushTo, &TransportState, &Children), Without<SensorPosition>>,
    sensors: Query<(&NodeId, &DioPin), (With<SensorPosition>, With<Switch>)>,
    io: Res<IoDevices>,
) {
    let filter = pushers
        .iter()
        .filter(|(.., state, _)| **state == TransportState::Sending)
        .filter_map(|(pusher, to, _, children)| {
            let any_pos_sensor_active = children
                .iter()
                .filter_map(|child| sensors.get(child).ok())
                .filter_map(|(node, pin)| io.get_input_bit(*node, *pin))
                .any(|switch| switch);
            if any_pos_sensor_active {
                None
            } else {
                Some((pusher, to.0))
            }
        });

    for (pusher, reciver) in filter {
        cmd.trigger(StopRunning(pusher));
        cmd.trigger(ShiftOver {
            from: pusher,
            to: reciver,
        });
    }
}

fn on_sensor_pos(
    mut trigger: On<PositionReached>,
    mut cmd: Commands,
    directions: Query<(&Direction, &TransportState)>,
) {
    let Ok((bana_dir, &state)) = directions.get(trigger.entity) else {
        return;
    };
    if !(state == TransportState::Reciving) {
        return;
    }

    trigger.propagate(false);
    let entity = trigger.entity;
    match (bana_dir, trigger.position) {
        (Direction::Forward, SensorPosition::LimitFront) => cmd.trigger((StopRunning(entity))),
        (Direction::Reverse, SensorPosition::LimitBack) => cmd.trigger(StopRunning(entity)),
        // (Direction::Reverse, SensorPosition::ProximityBack) => todo!(),
        // (Direction::Forward, SensorPosition::ProximityFront) => todo!(),
        _ => (),
    }
}

fn request_push(
    pushers: Query<(Entity, &PushTo, &TransportState)>,
    mut writer: MessageWriter<PushRequest>,
) {
    let filter_map = pushers.iter().filter_map(|(from, pushto, state)| {
        if state == &TransportState::ReadySend {
            Some((from, pushto.0))
        } else {
            None
        }
    });
    for (from, to) in filter_map {
        writer.write(PushRequest { from, to });
    }
}

fn push_request_handler(
    mut push_requests: MessageReader<PushRequest>,
    q: Query<&TransportState>,
    mut cmd: Commands,
) {
    for push in push_requests.read() {
        let Ok(_) = q.get(push.from) else {
            continue;
        };
        let Ok(reciver_state) = q.get(push.to) else {
            continue;
        };
        if !(reciver_state == &TransportState::ReadyRecive) {
            continue;
        }
        cmd.trigger(StartSending { entity: push.from });
        cmd.trigger(StartRecive(push.to));
    }
}

fn on_start_sending(
    trigger: On<StartSending>,
    mut banor: Query<(&mut TransportState, &Children, &Direction), Without<Movimot>>,
    motors: Query<&Movimot>,
    mut io: ResMut<IoDevices>,
) {
    let Ok((mut state, children, direction)) = banor.get_mut(trigger.entity) else {
        return;
    };
    *state = TransportState::Sending;
    for motor in children.iter().filter_map(|e| motors.get(e).ok()) {
        let (dio_set, dio_reset) = match direction {
            Direction::Forward => (motor.dq.forward, motor.dq.reverse),
            Direction::Reverse => (motor.dq.reverse, motor.dq.forward),
        };
        io.set_output_bit(dio_set.node, dio_set.pin, true);
        io.set_output_bit(dio_reset.node, dio_reset.pin, false);
    }
}
fn on_start_reciving(
    trigger: On<StartRecive>,
    mut banor: Query<(&mut TransportState, &Children, &Direction), Without<Movimot>>,
    motors: Query<&Movimot>,
    mut io: ResMut<IoDevices>,
) {
    let Ok((mut state, children, direction)) = banor.get_mut(trigger.0) else {
        return;
    };
    *state = TransportState::Reciving;
    for motor in children.iter().filter_map(|e| motors.get(e).ok()) {
        let (dio_set, dio_reset) = match direction {
            Direction::Forward => (motor.dq.forward, motor.dq.reverse),
            Direction::Reverse => (motor.dq.reverse, motor.dq.forward),
        };
        io.set_output_bit(dio_set.node, dio_set.pin, true);
        io.set_output_bit(dio_reset.node, dio_reset.pin, false);
    }
}

fn on_stop_running_tbana(
    trigger: On<StopRunning>,
    mut banor: Query<(&mut TransportState, &Children)>,
    mut cmd: Commands,
) {
    let Ok((mut state, children)) = banor.get_mut(trigger.0) else {
        return;
    };
    *state = TransportState::NotReady;
    for child in children {
        cmd.trigger(StopRunning(*child));
    }
}

fn on_stop_running_motor(
    trigger: On<StopRunning>,
    motors: Query<&Movimot>,
    mut io: ResMut<IoDevices>,
) {
    let Ok(motor) = motors.get(trigger.0) else {
        return;
    };
    let dios = [motor.dq.forward, motor.dq.reverse];
    for Dio { node, pin } in dios.into_iter() {
        io.set_output_bit(node, pin, false);
    }
}

fn set_tbana_ready(
    mut tbana: Query<(&mut TransportState, &RegisterPosition), With<NoProcess>>,
    reg: Res<Register>,
) {
    for (mut state, index) in tbana
        .iter_mut()
        .filter(|(state, _)| (state.as_ref()) == &TransportState::NotReady)
    {
        if reg.details[index.0 as usize].is_some() {
            *state = TransportState::ReadySend;
        } else {
            *state = TransportState::ReadyRecive;
        }
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
                let fw_node = motor.dq.forward.node;
                let fw_pin = motor.dq.forward.pin;
                let rev_node = motor.dq.reverse.node;
                let rev_pin = motor.dq.reverse.pin;
                let fw = io.get_output_bit(fw_node, fw_pin);
                let rev = io.get_output_bit(motor.dq.reverse.node, motor.dq.reverse.pin);
                let rapid =
                    io.get_output_bit(motor.dq.rapid.node, motor.dq.rapid.pin) == Some(true);
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
    let fc_names = ["back_end", "back_slow", "fron_slow", "front_end"];
    let fc_roles = [
        SensorPosition::LimitBack,
        SensorPosition::ProximityBack,
        SensorPosition::ProximityFront,
        SensorPosition::LimitFront,
    ];
    let io_inputs = spawn.io_inputs.iter();
    let phys_layers = CollisionLayers::new(PhysLayer::Sensor, PhysLayer::Detail);
    let fotocells: Vec<_> = io_inputs
        .zip(z_values)
        .zip(fc_names)
        .zip(fc_roles)
        .map(|(((dio, z), name), role)| {
            let coord = Vec3 {
                x: 0.45,
                y: 0.53,
                z,
            };
            let mut transform = Transform::from_translation(coord);
            transform.rotate_local_y(-90_f32.to_radians());
            let fotocell = FotocellBundle::new(name, dio.pin, &fotocell_assets, dio.node, 0.8);
            cmd.spawn((fotocell, transform, phys_layers, role))
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
            cmd.spawn((bundle, transform, phys_layers))
                .observe(on_stop_running_motor)
                .id()
        })
        .collect();

    let bana_bundle = (
        TbanaBundle::new(&tbana_assets),
        spawn.transform,
        Name::new(spawn.name.clone()),
        spawn.direction,
        spawn.register_pos,
    );

    let mut tbana = cmd.entity(spawn.entity);
    tbana
        .insert(bana_bundle)
        .add_children(&fotocells[0..4])
        .add_children(&motors_wheels[0..2])
        .observe(on_stop_running_tbana)
        .observe(on_start_reciving)
        .observe(on_sensor_pos)
        .observe(on_start_sending);

    if let Some(parrent) = spawn.parrent {
        tbana.insert(ChildOf(parrent));
    }

    if let Some(pushto) = spawn.push_to {
        tbana.insert(pushto);
    }
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
    pub ready: TransportState,
    simple: NoProcess,
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
            simple: NoProcess,
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
            node: address,
            pin: DioPin(fw),
        };
        let reverse = Dio {
            node: address,
            pin: DioPin(rev),
        };

        let rapid = Dio {
            node: address,
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

#[derive(Component, Debug, Clone, Copy, Reflect, Default, PartialEq, Eq)]
pub enum RunState {
    Ready,
    Running,
    #[default]
    Disabled,
}

#[derive(Component, Debug, Clone, Copy, Reflect, Default, PartialEq, Eq)]
pub enum TransportState {
    ReadySend,
    Sending,
    ReadyRecive,
    Reciving,
    #[default]
    NotReady,
}

#[derive(Message)]
pub struct PushRequest {
    pub from: Entity,
    pub to: Entity,
}

#[derive(EntityEvent)]
pub struct StartSending {
    pub entity: Entity,
}

#[derive(EntityEvent)]
pub struct StartRecive(pub Entity);

#[derive(EntityEvent)]
pub struct StopRunning(pub Entity);

#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub enum Mode {
    #[default]
    Push,
    Pull,
}

#[derive(Component, Reflect, Clone, Copy, Deref)]
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

#[derive(Component)]
/// Mark station as place where the product is *not* worked on
/// for example a buffer place or transportation station
pub struct NoProcess;
