use std::borrow::Cow;

use bevy::{color::palettes::css, prelude::*};

use crate::{
    io::{Address, ConnectedTo, DeviceNetwork, IoSlot},
    tbana::load_assets,
};

pub struct FotocellPlugin;

impl Plugin for FotocellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FotocellAssets>();
        app.add_systems(Startup, load_fotocell_assets.before(load_assets));
    }
}

fn load_fotocell_assets(
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut fotocell_assets: ResMut<FotocellAssets>,
) {
    fotocell_assets.emmiter = mesh_assets.add(Extrusion::new(Rectangle::new(0.02, 0.02), 0.14));
    fotocell_assets.laser = mesh_assets.add(Extrusion::new(Circle::new(0.005), 1.0));

    fotocell_assets.foto_materials.emmiter = material_assets.add(StandardMaterial {
        base_color: css::HOT_PINK.into(),
        ..Default::default()
    });
    fotocell_assets.foto_materials.reflector = material_assets.add(StandardMaterial {
        base_color: css::LAVENDER.into(),
        ..Default::default()
    });
    fotocell_assets.foto_materials.laser_normal = material_assets.add(StandardMaterial {
        base_color: css::ORCHID.into(),
        ..Default::default()
    });
    fotocell_assets.foto_materials.laser_triggerd = material_assets.add(StandardMaterial {
        base_color: css::LIME.into(),
        ..Default::default()
    });
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
    laser_normal: Handle<StandardMaterial>,
    laser_triggerd: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct FotocellAssets {
    emmiter: Handle<Mesh>,
    laser: Handle<Mesh>,
    reflector: Handle<Mesh>,
    foto_materials: FotoCellMaterials,
}

#[derive(Component)]
pub struct Fotocell;

// Fotocell,
// Name::new(format!("fotocell{i}")),
// IoSlot::new(ptr, io::DataSlice::Bit(idx)),
