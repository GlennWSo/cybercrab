use std::borrow::Cow;

use avian3d::prelude::*;
use bevy::{
    color::palettes::css,
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_polyline::{material::PolylineMaterialHandle, polyline::PolylineHandle, prelude::*};
use bitvec::vec::BitVec;

use crate::{
    io::{ConnectedTo, DIOPin, IoDevices, NetAddress, Switch},
    sysorder::InitSet,
};

pub struct FotocellPlugin;

impl Plugin for FotocellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PolylinePlugin);
        app.init_resource::<FotocellAssets>();
        app.add_systems(Startup, load_fotocell_assets.in_set(InitSet::LoadAssets));
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
    mut polylines_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut fotocell_assets: ResMut<FotocellAssets>,
) {
    fotocell_assets.emmiter = mesh_assets.add(Extrusion::new(
        Rectangle::new(0.02, 0.02),
        LASER_VERTS[0].z - 0.01,
    ));
    fotocell_assets.laser_poly = polylines.add(Polyline {
        vertices: LASER_VERTS.into(),
    });
    material_assets.add(StandardMaterial {
        base_color: css::HOT_PINK.into(),
        ..Default::default()
    });
    fotocell_assets.foto_materials.reflector = material_assets.add(StandardMaterial {
        base_color: css::LAVENDER.into(),
        ..Default::default()
    });
    let width = 2.0;
    fotocell_assets.foto_materials.laser_normal = polylines_materials.add(PolylineMaterial {
        color: css::ORCHID.into(),
        width,
        perspective: false,
        ..Default::default()
    });
    fotocell_assets.foto_materials.laser_triggerd = polylines_materials.add(PolylineMaterial {
        color: css::LIME.into(),
        width,
        perspective: false,
        ..Default::default()
    });
}

#[derive(Component)]
pub struct DetectorRay;

#[derive(Bundle)]
pub struct LaserBundle {
    pub poly: PolylineBundle,
    pub name: Name,
}

pub fn on_fotocell_blocked(
    trigger: Trigger<OnCollisionStart>,
    query: Query<(&Children, &ConnectedTo, &DIOPin)>,
    mut io: ResMut<IoDevices>,
    mut cmd: Commands,
) {
    let Ok((children, connection, pin)) = query.get(trigger.target()) else {
        return;
    };
    for child in children {
        cmd.entity(*child).trigger(SetButtonColor::Pressed);
    }
    let Some(data) = io.inputs.get_mut(&connection.0) else {
        println!("no such address:{}", connection.0);
        return;
    };
    dbg!(pin);
    data.as_mut_bitslice().set(**pin as usize, true);
}
pub fn on_fotocell_unblocked(
    trigger: Trigger<OnCollisionEnd>,
    query: Query<(&Children, &ConnectedTo, &DIOPin)>,
    mut io: ResMut<IoDevices>,
    mut cmd: Commands,
) {
    let Ok((children, connection, pin)) = query.get(trigger.target()) else {
        return;
    };
    for child in children {
        cmd.entity(*child).trigger(SetButtonColor::Released);
    }
    let Some(data) = io.inputs.get_mut(&connection.0) else {
        println!("no such address:{}", connection.0);
        return;
    };
    data.as_mut_bitslice().set(**pin as usize, false);
}

pub fn on_laser_color(
    trigger: Trigger<SetButtonColor>,
    mut qlaser: Query<&mut PolylineMaterialHandle>,
    assets: Res<FotocellAssets>,
) {
    let Ok(mut material) = qlaser.get_mut(trigger.target()) else {
        return;
    };
    match trigger.event() {
        SetButtonColor::Pressed => material.0 = assets.foto_materials.laser_triggerd.clone(),
        SetButtonColor::Released => material.0 = assets.foto_materials.laser_normal.clone(),
    }
}

#[derive(Event)]
pub enum SetButtonColor {
    Pressed,
    Released,
}

impl LaserBundle {
    pub fn new(assets: &FotocellAssets) -> Self {
        let polyline = PolylineHandle(assets.laser_poly.clone());
        let material = PolylineMaterialHandle(assets.foto_materials.laser_normal.clone());
        let poly = PolylineBundle {
            polyline,
            material,
            ..default()
        };
        Self {
            poly,
            name: Name::new("Fotocell Laser"),
        }
    }
}

// fn spawn_fotocell(cmd: &mut Commands, coord: Vec3, io_device: Address, slot: Slot) {}
#[derive(Bundle)]
pub struct FotocellBundle {
    pub fotocell_mark: Fotocell,
    pub switch: Switch,
    pub name: Name,
    pub device: ConnectedTo,
    pub io_pin: DIOPin,
    pub mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    simbody: RigidBody,
    collider: Collider,
    collision_marker: CollisionEventsEnabled,
}

impl FotocellBundle {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        io_slot: DIOPin,
        fotocell_assets: &FotocellAssets,
        io_device: NetAddress,
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
            fotocell_mark: Fotocell,
            name: Name::new(name),
            io_pin: io_slot,
            device: ConnectedTo(io_device),
            mesh: Mesh3d(fotocell_assets.emmiter.clone()),
            material: MeshMaterial3d(fotocell_assets.foto_materials.emmiter.clone()),
            switch: default(),
            simbody: RigidBody::Kinematic,
            collision_marker: CollisionEventsEnabled,
            collider,
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
    laser_normal: Handle<PolylineMaterial>,
    laser_triggerd: Handle<PolylineMaterial>,
}

#[derive(Resource, Default)]
pub struct FotocellAssets {
    emmiter: Handle<Mesh>,
    laser_poly: Handle<Polyline>,
    reflector: Handle<Mesh>,
    foto_materials: FotoCellMaterials,
}

#[derive(Component)]
pub struct Fotocell;

// Fotocell,
// Name::new(format!("fotocell{i}")),
// IoSlot::new(ptr, io::DataSlice::Bit(idx)),
