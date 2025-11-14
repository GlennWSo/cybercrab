use std::iter::once;

use avian3d::prelude::*;
use bevy::prelude::{MeshMaterial3d, *};
use itertools::Itertools;

use crate::{
    fotocell::{FotocellAssets, FotocellBundle},
    io::{FreePins, InputNode, WiredTo},
    tbana::{PushTo, TBanaAssets, TbanaBundle},
};

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut inputs: Query<(Entity, &mut FreePins), With<InputNode>>,
    mut outputs: Query<(Entity, &mut FreePins), With<InputNode>>,
) -> Result {
    let required_out_pins = 3;
    let required_in_pins = 2;

    let (input_module, mut in_pins) = inputs
        .iter_mut()
        .find(|(_id, free_pins)| free_pins.len() < required_out_pins)
        .ok_or("Not enough input slots")?;

    let (output_module, mut out_pins) = outputs
        .iter_mut()
        .find(|(_id, free_pins)| free_pins.len() < required_out_pins)
        .ok_or("Not enough output slots")?;

    let foto_pins = in_pins.take_pins(required_in_pins).unwrap();
    let motor_pins = out_pins.take_pins(required_out_pins).unwrap();

    let sensors = trigger
        .fotocell_positions
        .into_iter()
        .zip(foto_pins.iter())
        .map(|(pos, &pin)| {
            let fotocell =
                FotocellBundle::new(format!("Fotocell {}", pin.0), &fotocell_assets, 0.8);
            let transform = Transform::from_translation(Vec3 {
                x: 0.45,
                y: 0.53,
                z: pos,
            });
            let foto_cell_bundle = (fotocell, transform, WiredTo(input_module), pin);
            cmd.spawn(foto_cell_bundle).id()
        })
        .collect_vec();

    let motor_bundle = (
        Name::new("Movimot"),
        WiredTo(output_module),
        motor_pins,
        Transform::from_translation(Vec3 {
            y: 0.5,
            ..default()
        }),
    );

    let wheels = trigger
        .wheel_positions
        .into_iter()
        .enumerate()
        .map(|(i, z)| {
            let material = MeshMaterial3d(tbana_assets.wheel_material.clone());
            let bundle = TranportWheel::new(&mut meshes, material);
            let transform = Transform::from_translation(Vec3 { z, ..default() });
            cmd.spawn((bundle, transform, Name::new(format!("Wheel{i}"))))
                .id()
        })
        .collect_vec();

    let motor = cmd.spawn(motor_bundle).add_children(&wheels).id();

    let children = sensors.into_iter().chain(once(motor)).collect_vec();

    let bana_bundle = (TbanaBundle::new(&tbana_assets), trigger.transform);
    let mut bana = cmd.spawn(bana_bundle);
    bana.add_children(&children);

    if let Some(pushto) = trigger.pushto {
        bana.insert(pushto);
    }
    Ok(())
}
