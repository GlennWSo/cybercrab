use std::borrow::Cow;

use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
// use bevy_polyline::{material::PolylineMaterialHandle, polyline::PolylineHandle, prelude::*};

use crate::{
    io::{Memory, PinIndex, Switch, SwitchSet, WiredTo},
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

pub fn on_fotocell_blocked(
    trigger: On<CollisionStart>,
    mut cmd: Commands,
    fotocells: Query<(&WiredTo, &PinIndex), With<SensorRange>>,
) {
    let fotocell_id = trigger.event_target();
    let Ok((wire, pin)) = fotocells.get(fotocell_id) else {
        return;
    };
    cmd.trigger(SwitchSet {
        target: wire.0,
        slot: *pin,
        value: true,
    });
}

pub fn on_fotocell_unblocked(
    trigger: On<CollisionEnd>,
    mut cmd: Commands,
    fotocells: Query<(&WiredTo, &PinIndex)>,
) {
    let fotocell_id = trigger.event_target();
    let Ok((wire, pin)) = fotocells.get(fotocell_id) else {
        return;
    };
    cmd.trigger(SwitchSet {
        target: wire.0,
        slot: *pin,
        value: false,
    });
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct DetectorGizmos;

fn render_fotocell_detector(
    mut gizmos: Gizmos<DetectorGizmos>,
    q: Query<(&SensorRange, &GlobalTransform, &WiredTo, &PinIndex)>,
    io: Query<&Memory>,
) {
    for (fc, transform, wire, pin) in q {
        let start = transform.translation();
        let end = start - transform.forward() * fc.0;
        let Ok(memory) = io.get(wire.0) else {
            continue;
        };

        let color = match memory.get(**pin as usize) {
            Some(bit) if *bit => css::GREEN,
            Some(_) => css::PURPLE,
            None => css::DARK_GRAY,
        };

        gizmos.line(start, end, color);
    }
}

// fn spawn_fotocell(cmd: &mut Commands, coord: Vec3, io_device: Address, slot: Slot) {}
#[derive(Bundle)]
pub struct FotocellBundle {
    fotocell: SensorRange,
    switch: Switch,

    pub name: Name,
    pub mesh: Mesh3d,
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
            fotocell: SensorRange(range),
            name: Name::new(name),
            mesh: Mesh3d(fotocell_assets.emmiter.clone()),
            material: MeshMaterial3d(fotocell_assets.foto_materials.emmiter.clone()),
            simbody: RigidBody::Kinematic,
            collision_marker: CollisionEventsEnabled,
            collider,
            switch: default(),
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

#[derive(Component, Reflect)]
/// Adds fotocell behaivor to a Entity
pub struct SensorRange(f32);

impl SensorRange {
    pub fn new(range: f32) -> Self {
        Self(range)
    }
}

// Fotocell,
// Name::new(format!("fotocell{i}")),
// IoSlot::new(ptr, io::DataSlice::Bit(idx)),
