use std::sync::Arc;

use avian3d::{collision::collider, prelude::*};
use bevy::{color::palettes::css, prelude::*};

use crate::InitSet;

pub struct ShiftRegPlugin;

impl Plugin for ShiftRegPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Slot>();
        app.init_resource::<DetailAssets>();
        app.add_systems(Startup, load_assets.in_set(InitSet::LoadAssets));
        app.add_systems(Startup, spawn_test_detail.in_set(InitSet::Spawn));
    }
}

fn spawn_test_detail(mut cmd: Commands, assets: Res<DetailAssets>) {
    let bundle = (
        DetailBundle::new(&assets),
        Name::new("Detail_1"),
        RigidBody::Kinematic,
    );
    cmd.spawn(bundle);
}

fn load_assets(
    mut detail_resource: ResMut<DetailAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    detail_resource.base_shape = meshes.add(Extrusion::new(Rectangle::new(0.3, 0.3), 1.5));
    detail_resource.normal_material = materials.add(StandardMaterial {
        base_color: css::DARK_OLIVEGREEN.into(),
        ..Default::default()
    });
    detail_resource.collider = Collider::cuboid(0.3, 0.3, 1.5);
}

#[derive(Component)]
pub struct Detail;

#[derive(Bundle)]
pub struct DetailBundle {
    pub transform: Transform,
    marker: Detail,
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    collider: Collider,
}

impl DetailBundle {
    pub fn new(assets: &DetailAssets) -> Self {
        Self {
            marker: Detail,
            transform: Transform::default(),
            mesh: Mesh3d(assets.base_shape.clone()),
            material: MeshMaterial3d(assets.normal_material.clone()),
            collider: assets.collider.clone(),
        }
    }
}

#[derive(Resource, Default)]
pub struct DetailAssets {
    base_shape: Handle<Mesh>,
    normal_material: Handle<StandardMaterial>,
    collider: Collider, // TODO turn into Asset/Handle
}

#[derive(Component, Deref, Reflect, Default)]
pub struct Slot {
    pub detail: Option<Entity>,
}
