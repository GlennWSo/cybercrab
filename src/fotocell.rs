use std::borrow::Cow;

use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
// use bevy_polyline::{material::PolylineMaterialHandle, polyline::PolylineHandle, prelude::*};

use crate::{
    io::{DioPin, Io, IoDevices, Ip4, PinTerminals, Switch, SwitchSet},
    sensor::{on_sensor_switch, SensorPosition},
    sysorder::InitSet,
};

pub struct FotocellPlugin;

impl Plugin for FotocellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FotocellAssets>();
        app.init_gizmo_group::<DetectorGizmos>();
        app.add_systems(Startup, load_fotocell_assets.in_set(InitSet::LoadAssets));
        app.add_observer(on_sensor_switch);
        app.register_type::<SensorPosition>();
        app.add_systems(Update, render_fotocell_detector);
    }
}

const LASER_VERTS: [Vec3; 2] = [
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.15,
    },
    Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.7,
    },
];
fn load_fotocell_assets(
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    // mut polylines_materials: ResMut<Assets<PolylineMaterial>>,
    // mut polylines: ResMut<Assets<Polyline>>,
    mut fotocell_assets: ResMut<FotocellAssets>,
) {
    fotocell_assets.emmiter = mesh_assets.add(Extrusion::new(
        Rectangle::new(0.02, 0.02),
        LASER_VERTS[0].z - 0.01,
    ));
    material_assets.add(StandardMaterial {
        base_color: css::HOT_PINK.into(),
        ..Default::default()
    });
    fotocell_assets.foto_materials.reflector = material_assets.add(StandardMaterial {
        base_color: css::LAVENDER.into(),
        ..Default::default()
    });
}

#[derive(Component)]
pub struct DetectorRay;

pub fn on_fotocell_blocked(trigger: On<CollisionStart>, mut cmd: Commands) {
    cmd.trigger(SwitchSet {
        entity: trigger.event_target(),
        closed: true,
        kind: Io::Input,
    });
}

pub fn on_fotocell_unblocked(trigger: On<CollisionEnd>, mut cmd: Commands) {
    cmd.trigger(SwitchSet {
        entity: trigger.event_target(),
        closed: false,
        kind: Io::Input,
        PhysicsInterpolationPlugin,
    });
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct DetectorGizmos;

fn render_fotocell_detector(
    mut gizmos: Gizmos<DetectorGizmos>,
    q: Query<(&Fotocell, &GlobalTransform, &Ip4, &DioPin)>,
    devices: Res<IoDevices>,
) {
    for (fc, transform, address, pin) in q {
        let start = transform.translation();
        let end = start - transform.forward() * fc.range;

        let color = match devices.digital_inputs.get(address) {
            Some(device) => match device.get(pin.as_usize()) {
                Some(true) => css::GREEN,
                Some(false) => css::PURPLE,
                None => css::GRAY, // no value at pin number
            },
            None => css::DARK_GRAY, // no device
        };
        // let color = match devices.digital_inputs.get(k)

        gizmos.line(start, end, color);
    }
}

// fn spawn_fotocell(cmd: &mut Commands, coord: Vec3, io_device: Address, slot: Slot) {}
#[derive(Bundle)]
pub struct FotocellBundle {
    pub fotocell_mark: Fotocell,
    pub switch: Switch,
    pub name: Name,
    pub mesh: Mesh3d,
    pub pins: PinTerminals,
    material: MeshMaterial3d<StandardMaterial>,
    simbody: RigidBody,
    collider: Collider,
    collision_marker: CollisionEventsEnabled,
}

impl FotocellBundle {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        fotocell_assets: &FotocellAssets,
        range: f32,
    ) -> Self {
        let collider = Collider::segment(
            Vec3::ZERO,
            Vec3 {
                z: range,
                ..default()
            },
        );
        Self {
            fotocell_mark: Fotocell { range },
            name: Name::new(name),
            mesh: Mesh3d(fotocell_assets.emmiter.clone()),
            material: MeshMaterial3d(fotocell_assets.foto_materials.emmiter.clone()),
            switch: default(),
            simbody: RigidBody::Kinematic,
            collision_marker: CollisionEventsEnabled,
            collider,
            pins: PinTerminals::default(),
        }
    }
    pub fn with_translation(self, translation: Vec3) -> (Self, Transform) {
        (self, Transform::from_translation(translation))
    }
}

#[derive(Default)]
struct FotoCellMaterials {
    emmiter: Handle<StandardMaterial>,
    reflector: Handle<StandardMaterial>,
    // laser_normal: Handle<PolylineMaterial>,
    // laser_triggerd: Handle<PolylineMaterial>,
}

#[derive(Resource, Default)]
pub struct FotocellAssets {
    emmiter: Handle<Mesh>,
    // laser_poly: Handle<Polyline>,
    reflector: Handle<Mesh>,
    foto_materials: FotoCellMaterials,
}

#[derive(Component)]
/// Adds fotocell behaivor to a Entity
pub struct Fotocell {
    range: f32,
}

impl Fotocell {
    pub fn new(range: f32) -> Self {
        Self { range }
    }
}

// Fotocell,
// Name::new(format!("fotocell{i}")),
// IoSlot::new(ptr, io::DataSlice::Bit(idx)),
