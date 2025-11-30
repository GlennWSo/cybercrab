use std::{borrow::Cow, iter::once};

use avian3d::prelude::*;
use bevy::prelude::{MeshMaterial3d, *};
use itertools::Itertools;

use crate::{
    fotocell::{FotocellAssets, FotocellBundle},
    io::{
        FreePins, InputDevice, InputNodeBundle, OutputDevice, OutputNodeBundle, PinnedTo, WiredTo,
    },
    motor::{MotorAssets, WheelBundle},
    tbana::{PushTo, TBanaAssets, TbanaBundle},
};

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawn_tbana)
            .add_systems(Startup, setup_tbana);
    }
}

pub enum SpawnLocation {
    Before(Entity),
    After(Entity),
    Origin(Transform),
}

impl Default for SpawnLocation {
    fn default() -> Self {
        Self::Origin(Transform::default())
    }
}

#[derive(Event)]
pub struct SpawnTBana {
    pub fotocell_positions: [f32; 4],
    pub wheel_positions: [f32; 3],
    pub location: SpawnLocation,
    pub input: PinnedTo<4>,
    pub output: PinnedTo<3>,
    pub name: Cow<'static, str>,
}

impl SpawnTBana {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        input: PinnedTo<4>,
        output: PinnedTo<3>,
    ) -> Self {
        Self {
            name: name.into(),
            fotocell_positions: [-0.9, -0.7, 0.7, 0.9],
            wheel_positions: [-0.8, 0.0, 0.8],
            location: Default::default(),
            input,
            output,
        }
    }
}

// impl Default for SpawnTBana {
//     fn default() -> Self {
//         Self {
//             fotocell_positions: [-0.9, -0.7, 0.7, 0.9],
//             wheel_positions: [-0.8, 0.0, 0.8],
//             location: Default::default(),
//             ..todo!()
//         }
//     }
// }

fn setup_tbana(mut cmd: Commands) {
    let mut input_bundle = InputNodeBundle::new(crate::io::Address(0), 64);
    let mut output_bundle = OutputNodeBundle::new(crate::io::Address(1), 64);
    dbg!(&input_bundle.free_pins);
    let input_pins = input_bundle.free_pins.take_pins().unwrap();
    let output_pins = output_bundle.free_pins.take_pins().unwrap();
    let input_id = cmd.spawn(input_bundle).id();
    let output_id = cmd.spawn(output_bundle).id();

    let input = PinnedTo {
        to: input_id,
        pins: input_pins,
    };
    let output = PinnedTo {
        to: output_id,
        pins: output_pins,
    };

    let event = SpawnTBana::new("Bana1", input, output);
    cmd.trigger(event);
}

pub fn on_spawn_tbana(
    trigger: On<SpawnTBana>,
    mut cmd: Commands,
    tbana_assets: Res<TBanaAssets>,
    fotocell_assets: Res<FotocellAssets>,
    motor_assets: Res<MotorAssets>,

    transforms: Query<&Transform>,
) -> Result {
    let transform = match trigger.location {
        SpawnLocation::Origin(transform) => transform,
        SpawnLocation::Before(entity) | SpawnLocation::After(entity) => *transforms.get(entity)?,
    };

    let sensors = trigger
        .fotocell_positions
        .into_iter()
        .zip(trigger.input.pins.iter())
        .map(|(pos, &pin)| {
            let fotocell =
                FotocellBundle::new(format!("Fotocell {}", pin.0), &fotocell_assets, 0.8);
            let transform = Transform::from_translation(Vec3 {
                x: 0.45,
                y: 0.53,
                z: pos,
            })
            .with_rotation(Quat::from_rotation_y(-90_f32.to_radians()));
            let foto_cell_bundle = (fotocell, transform, WiredTo(trigger.input.to), pin);
            cmd.spawn(foto_cell_bundle).id()
        })
        .collect_vec();

    let motor_bundle = (
        Name::new("Movimot"),
        WiredTo(trigger.output.to),
        trigger.output.pins,
        Transform::from_translation(Vec3 {
            y: 0.47,
            ..default()
        }),
    );

    let wheels = trigger
        .wheel_positions
        .into_iter()
        .enumerate()
        .map(|(i, z)| {
            let bundle = WheelBundle::new(&motor_assets);
            let transform = Transform::from_translation(Vec3 { z, ..default() })
                .with_rotation(Quat::from_rotation_z(90_f32.to_radians()));
            cmd.spawn((bundle, transform, Name::new(format!("Wheel{i}"))))
                .id()
        })
        .collect_vec();

    let motor = cmd.spawn(motor_bundle).add_children(&wheels).id();

    let children = sensors.into_iter().chain(once(motor)).collect_vec();

    let bana_bundle = (TbanaBundle::new(&tbana_assets), transform);
    let mut bana = cmd.spawn((Name::new(trigger.name.clone()), bana_bundle));
    bana.add_children(&children);

    if let SpawnLocation::Before(entity) = trigger.location {
        bana.insert(PushTo(entity));
    }
    Ok(())
}
