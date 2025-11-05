use bevy::prelude::*;
use itertools::Itertools;

use crate::io::{DIOModule, Dio, InputBundle, OutputBundle};

pub fn setup_tbana(input_module: &mut DIOModule, output_module: &mut DIOModule) {
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
}
