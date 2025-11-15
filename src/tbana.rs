use std::borrow::Cow;

use avian3d::prelude::{Collider, CollidingEntities, CollisionLayers, LinearVelocity};
use bevy::color::palettes::css;
use bevy::prelude::{Mesh3d, *};

use crate::fotocell::{on_switch_collsion_end, FotocellAssets, FotocellBundle, SensorRange};
use crate::io::{Address, Memory, PinIndex, Switch, WiredTo};
use crate::physics::PhysLayer;
use crate::sensor::{
    FrontLimit, FrontProximity, PositionReached, RearLimit, RearProximity, SensorPosition,
};
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
        // app.register_type::<Movimot>();
        app.add_message::<PushRequest>();
        app.init_resource::<TBanaAssets>();
        app.add_systems(Startup, load_assets.in_set(InitSet::LoadAssets));
        app.add_systems(
            Update,
            (
                // motor_effect,
                // tbana_motor_logic,
                push_request_handler,
                request_push,
                stop_pushing,
                set_tbana_ready,
            ),
        );
        // app.add_observer(on_insert_tbana);
        app.add_observer(on_switch_tbana_direction);
    }
}

#[derive(Component)]
/// Tbana with sensors for slowdown and reverisablity
pub struct FineReversableTbana {
    pub front_pos: Entity,
    pub front_proximity: Entity,
    pub back_pos: Entity,
    pub back_proximity: Entity,
}

#[derive(Bundle)]
pub struct TBanaBundle {
    pub auto: AutoMode,
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub mode: Mode,
    pub ready: TransportState,
}

impl TBanaBundle {
    pub fn new(tbana_assets: &TBanaAssets) -> Self {
        Self {
            auto: AutoMode::default(),
            mesh: Mesh3d(tbana_assets.base_mesh.clone()),
            material: MeshMaterial3d(tbana_assets.bana_material.clone()),
            mode: default(),
            ready: default(),
        }
    }
}

#[derive(Event)]
pub struct SwitchDirection;

fn on_switch_tbana_direction(
    trigger: On<SwitchDirection>,
    mut cmd: Commands,
    mut tbanor: Query<(
        Entity,
        &mut Reversiable,
        &mut TransportState,
        &PushTo,
        &RegisterPosition,
    )>,
    reg: Res<Register>,
) {
    for (me, mut dir, mut state, pushto, pos) in tbanor.iter_mut() {
        match *dir.as_ref() {
            Reversiable::Forward => *dir = Reversiable::Reverse,
            Reversiable::Reverse => *dir = Reversiable::Forward,
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

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
pub enum Reversiable {
    #[default]
    Forward,
    Reverse,
}

fn stop_pushing(
    mut cmd: Commands,
    banor: Query<(Entity, &TransportState)>,
    limit_sensors: Query<&Children, With<FrontLimit>>,
    proxmity_sensors: Query<&Children, With<FrontLimit>>,
    sensors: Query<(&WiredTo, &PinIndex)>,
    memory: Query<&Memory>,
) {
    for bana in banor.iter().filter_map(|(bana, state)| match state {
        TransportState::Sending => Some(bana),
        _ => None,
    }) {
        let limit_detected = limit_sensors
            .iter_descendants(bana)
            .flat_map(|id| sensors.get(id).ok())
            .flat_map(|(&id, &pin)| {
                memory
                    .get(id.0)
                    .map(|mem| mem.get(*pin as usize).map(|r| *r))
            })
            .flatten()
            .next();
    }

    // for pusher in filter {
    //     cmd.trigger(StopRunning(pusher));
    // }
}

fn on_sensor_pos(
    mut trigger: On<PositionReached>,
    mut cmd: Commands,
    directions: Query<(&Reversiable, &TransportState)>,
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
        (Reversiable::Forward, SensorPosition::LimitFront) => cmd.trigger((StopRunning(entity))),
        (Reversiable::Reverse, SensorPosition::LimitBack) => cmd.trigger(StopRunning(entity)),
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
    mut banor: Query<(&mut TransportState, &mut MovimotBits, &Reversiable)>,
) {
    let Ok((mut state, mut movimot, direction)) = banor.get_mut(trigger.entity) else {
        return;
    };

    match *state {
        TransportState::ReadySend => *state = TransportState::Sending,
        _ => return,
    }

    match direction {
        Reversiable::Forward => *movimot = MovimotBits::SlowForward,
        Reversiable::Reverse => *movimot = MovimotBits::SlowReverse,
    }
}
fn on_start_reciving(
    trigger: On<StartRecive>,
    mut banor: Query<(&mut TransportState, &mut MovimotBits, &Reversiable)>,
) {
    let Ok((mut state, mut movimot, direction)) = banor.get_mut(trigger.event_target()) else {
        return;
    };

    match *state {
        TransportState::ReadyRecive => *state = TransportState::Reciving,
        _ => return,
    }

    match direction {
        Reversiable::Forward => *movimot = MovimotBits::SlowForward,
        Reversiable::Reverse => *movimot = MovimotBits::SlowReverse,
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

fn set_tbana_ready(
    mut tbana: Query<(&mut TransportState, &RegisterPosition), With<NoProcess>>,
    reg: Res<Register>,
) {
    todo!()
}

// fn tbana_motor_effects(motors: Query<(&CollidingEntities, &MovimotDQ)>, io: Res<IoDevices>) {}

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
    // bana_res.bana_materials.ready = ready_mode;
    todo!();
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
    todo!();
    // tbana_res.wheel_materials.ready = ready_mode;
    // tbana_res.wheel_materials.running = run_mode;
    // tbana_res.wheel_materials.alarm = alarm_mode;

    tbana_res.base_mesh = mesh_assets.add(Extrusion::new(Rectangle::default(), 2.0));
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
    pub base_mesh: Handle<Mesh>,
    pub bana_material: Handle<StandardMaterial>,
    pub wheel_material: Handle<StandardMaterial>,
    pub wheel_mesh: Handle<Mesh>,
    pub wheel_collider: Collider,
}

#[derive(Component, Reflect)]
pub struct TransportBana;

#[derive(Reflect, Component, Default, Deref)]
pub struct AutoMode {
    enabled: bool,
}

#[derive(Component, Reflect)]
#[component(immutable)]
pub struct Radius(f32);

#[derive(Component, Reflect, Default)]
pub struct Wheel {
    /// surface tangent speed m/s
    speed: f32,
}

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
#[repr(u8)]
pub enum MovimotBits {
    #[default]
    Stop = 0b000,
    SlowForward = 0b010,
    SlowReverse = 0b100,
    FastForward = 0b011,
    FastReverse = 0b101,
}

#[derive(Component, Reflect, Debug)]
pub struct MovimotCfg {
    /// rotations per second
    pub fast_rps: f32,
    /// rotations per second`
    pub slow_rps: f32,
}

impl Default for MovimotCfg {
    fn default() -> Self {
        Self {
            fast_rps: 10.0,
            slow_rps: 2.0,
        }
    }
}

impl MovimotCfg {
    pub fn rpm(fast_rpm: f32, slow_rpm: f32) -> Self {
        Self {
            fast_rps: fast_rpm / 60.0,
            slow_rps: slow_rpm / 60.0,
        }
    }
}

#[derive(Bundle, Default)]
struct Movimot {
    io: MovimotBits,
    cfg: MovimotCfg,
}

#[derive(Bundle)]
pub struct WheelBundle {
    marker: Wheel,
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    collider: Collider,
}

impl WheelBundle {
    pub fn new(assets: &TBanaAssets, dq: MovimotBits) -> Self {
        todo!()
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
