use std::borrow::Cow;

use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bevy_polyline::{material::PolylineMaterialHandle, polyline::PolylineHandle, prelude::*};

use crate::{
    io::{ConnectedTo, IoSlot},
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
struct DetectorRay;

#[derive(Bundle)]
pub struct LaserBundle {
    marker: DetectorRay,
    pub poly: PolylineBundle,
    pub name: Name,
    simbody: RigidBody,
    collider: Collider,
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
            marker: DetectorRay,
            poly,
            name: Name::new("Fotocell Laser"),
            simbody: RigidBody::Static,
            collider: Collider::polyline(LASER_VERTS.into(), None),
        }
    }
}

// fn spawn_fotocell(cmd: &mut Commands, coord: Vec3, io_device: Address, slot: Slot) {}
#[derive(Bundle)]
pub struct FotocellBundle {
    pub marker: Fotocell,
    pub name: Name,
    pub io_slot: IoSlot,
    pub device: ConnectedTo,
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
}

impl FotocellBundle {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        io_slot: IoSlot,
        fotocell_assets: &FotocellAssets,
        io_device: Entity,
    ) -> Self {
        Self {
            marker: Fotocell,
            name: Name::new(name),
            io_slot,
            device: ConnectedTo(io_device),
            mesh: Mesh3d(fotocell_assets.emmiter.clone()),
            material: MeshMaterial3d(fotocell_assets.foto_materials.emmiter.clone()),
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
