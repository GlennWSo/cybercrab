use std::iter::{self, once};

use avian3d::prelude::*;
use bevy::prelude::{MeshMaterial3d, *};
use itertools::Itertools;

use crate::{
    fotocell::{FotocellAssets, FotocellBundle},
    io::{InputPinsTo, OutputPinsTo, PinTerminals},
    tbana::{PushTo, TBanaAssets, TbanaBundle, TransportWheelBundle},
};

#[derive(Resource)]
struct TBanaMaterials {
    station_material: Handle<StandardMaterial>,
    wheel_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct TransportMotor;

#[derive(Component)]
pub struct Movimot {
    pub speed1: f32,
    pub speed2: f32,
}

impl Default for Movimot {
    fn default() -> Self {
        Self {
            speed1: 1.0,
            speed2: 5.0,
        }
    }
}

/// A thing that (phyically) moves the transport target
/// examples Converbelt, Driving wheels, pushing-piston
#[derive(Component)]
pub struct TransportDriver;

#[derive(Component)]
struct Ratio(f32);

#[derive(Bundle)]
pub struct TranportWheel {
    mesh: Mesh3d,
    collider: Collider,
    rigid_body: RigidBody,
    driver: TransportDriver,
    material: MeshMaterial3d<StandardMaterial>,
}

impl TranportWheel {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        material: MeshMaterial3d<StandardMaterial>,
    ) -> Self {
        let radius = 0.2;
        let height = 0.5;
        let collider = Collider::cylinder(radius, height);
        let mesh = meshes.add(Cylinder::new(radius, height));
        Self {
            collider,
            rigid_body: RigidBody::Static,
            driver: TransportDriver,
            mesh: Mesh3d(mesh),
            material,
        }
    }
}

#[derive(Event)]
pub struct SpawnTBana {
    pub input_module: InputPinsTo,
    pub output_module: OutputPinsTo,
    pub pushto: Option<PushTo>,
    pub fotocell_positions: [f32; 2],
    pub wheel_positions: [f32; 3],
    pub transform: Transform,
}

pub fn on_spawn_tbana(
    trigger: On<SpawnTBana>,
    mut cmd: Commands,
    tbana_assets: Res<TBanaAssets>,
    fotocell_assets: Res<FotocellAssets>,
    materals: Res<TBanaMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let wheels = trigger
        .wheel_positions
        .into_iter()
        .enumerate()
        .map(|(i, z)| {
            let material = MeshMaterial3d(materals.wheel_material.clone());
            let bundle = TranportWheel::new(&mut meshes, material);
            let transform = Transform::from_translation(Vec3 { z, ..default() });
            cmd.spawn((bundle, transform)).id()
        })
        .collect_vec();

    let motor_bundle = (
        Name::new("Movimot"),
        trigger.output_module,
        PinTerminals::new(3),
        Transform::from_translation(Vec3 {
            y: 0.5,
            ..default()
        }),
    );
    let motor = cmd.spawn(motor_bundle).add_children(&wheels).id();

    let sensors = trigger
        .fotocell_positions
        .into_iter()
        .enumerate()
        .map(|(i, z)| {
            let fotocell = FotocellBundle::new(format!("Fotocell {i}"), &fotocell_assets, 0.8);
            let transform = Transform::from_translation(Vec3 {
                x: 0.45,
                y: 0.53,
                z,
            });
            cmd.spawn((fotocell, transform, trigger.input_module)).id()
        });

    let children = sensors.chain(once(motor)).collect_vec();

    let bana_bundle = (TbanaBundle::new(&tbana_assets), trigger.transform);
    let mut bana = cmd.spawn(bana_bundle);
    bana.add_children(&children);

    if let Some(pushto) = trigger.pushto {
        bana.insert(pushto);
    }
}
