use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    fotocell::{FotocellAssets, FotocellBundle},
    io::{DIOModule, Dio, InputBundle, OutputBundle},
};

pub fn setup_tbana(
    cmd: &mut Commands,
    input_module: &mut DIOModule,
    output_module: &mut DIOModule,
    fotocell_assets: Res<FotocellAssets>,
) {
    let input_dios: [Dio; 2] = input_module
        .take(2)
        .collect_array()
        .expect("Input module must have available pins");
    let output_dios: [Dio; 3] = output_module
        .take(3)
        .collect_array()
        .expect("Output module must have available pins");

    let inputs = InputBundle::new(input_dios.as_ref().into());
    let outputs = OutputBundle::new(output_dios.as_ref().into());

    let sensor_positions = [0.9, 0.7];

    let sensors = sensor_positions
        .into_iter()
        .enumerate()
        .map(|(i, z)| {
            let fotocell = FotocellBundle::new(format!("PositionSensor{i}"), &fotocell_assets, 0.8);
            let coord = Vec3 {
                x: 0.45,
                y: 0.53,
                z,
            };
            let transform = Transform::from_translation(coord);
            (fotocell, transform)
        })
        .collect_vec();
}
