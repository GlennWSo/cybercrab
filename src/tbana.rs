use std::borrow::Cow;

use avian3d::prelude::Collider;
use bevy::color::palettes::css;
use bevy::prelude::{Mesh3d, *};

use crate::io::{DIOPin, DeviceAddress, Dio};
use crate::shiftreg::Slot;
use crate::sysorder::InitSet;

pub struct TbanaPlugin;

impl Plugin for TbanaPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AutoMode>();
        app.register_type::<PushTo>();
        app.register_type::<Reciver>();
        app.register_type::<PullFrom>();
        app.register_type::<Giver>();
        app.register_type::<Movimot>();
        app.init_resource::<TBanaAssets>();
        app.add_systems(Startup, load_assets.in_set(InitSet::LoadAssets));
    }
}

pub fn load_assets(
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut tbana_res: ResMut<TBanaAssets>,
) {
    let ready_mode = material_assets.add(StandardMaterial {
        base_color: css::CORNSILK.into(),
        ..Default::default()
    });
    let run_mode = material_assets.add(StandardMaterial {
        base_color: css::CHARTREUSE.into(),
        ..Default::default()
    });
    let alarm_mode = material_assets.add(StandardMaterial {
        base_color: css::CRIMSON.into(),
        ..Default::default()
    });
    tbana_res.bana_materials.ready = ready_mode;
    tbana_res.bana_materials.running = run_mode;
    tbana_res.bana_materials.alarm = alarm_mode;
    let shade = 0.2;
    let ready_mode = material_assets.add(StandardMaterial {
        base_color: css::CORNSILK.darker(shade).into(),
        ..Default::default()
    });
    let run_mode = material_assets.add(StandardMaterial {
        base_color: css::CHARTREUSE.darker(shade).into(),
        ..Default::default()
    });
    let alarm_mode = material_assets.add(StandardMaterial {
        base_color: css::CRIMSON.darker(shade).into(),
        ..Default::default()
    });
    tbana_res.wheel_materials.ready = ready_mode;
    tbana_res.wheel_materials.running = run_mode;
    tbana_res.wheel_materials.alarm = alarm_mode;

    tbana_res.bana_mesh = mesh_assets.add(Extrusion::new(Rectangle::default(), 2.0));
    tbana_res.wheel_mesh = mesh_assets.add(Extrusion::new(Circle::new(0.1), 0.6));
}

#[derive(Default)]
struct ModeMaterials {
    ready: Handle<StandardMaterial>,
    running: Handle<StandardMaterial>,
    alarm: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct TBanaAssets {
    bana_mesh: Handle<Mesh>,
    bana_materials: ModeMaterials,
    wheel_materials: ModeMaterials,
    wheel_mesh: Handle<Mesh>,
    wheel_collider: Collider,
}

#[derive(Bundle)]
pub struct TbanaBundle {
    pub tbana: TransportBana,
    pub name: Name,
    pub auto: AutoMode,
    pub slot: Slot,
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
}

impl TbanaBundle {
    pub fn new(name: impl Into<Cow<'static, str>>, tbana_assets: &TBanaAssets) -> Self {
        let name = Name::new(name);
        Self {
            name,
            tbana: TransportBana,
            auto: AutoMode::default(),
            slot: Slot::default(),
            mesh: Mesh3d(tbana_assets.bana_mesh.clone()),
            material: MeshMaterial3d(tbana_assets.bana_materials.ready.clone()),
        }
    }
}

#[derive(Component)]
pub struct TransportBana;

#[derive(Reflect, Component, Default, Deref)]
pub struct AutoMode {
    enabled: bool,
}

#[derive(Component)]
pub struct Wheel;

// enum MoviMotion {
//     FastForward,
//     Forward,
//     FastReverse,
//     Reverse,
//     Stop,
// }

#[derive(Component, Reflect)]
pub struct MovimotDQ {
    forward: Dio,
    reverse: Dio,
    rapid: Dio,
}

impl MovimotDQ {
    pub fn new(address: u32, fw: u16, rev: u16, rapid: u16) -> Self {
        let address = DeviceAddress(address);
        let forward = Dio {
            address,
            pin: DIOPin(fw),
        };
        let reverse = Dio {
            address,
            pin: DIOPin(rev),
        };

        let rapid = Dio {
            address,
            pin: DIOPin(rapid),
        };

        Self {
            forward,
            reverse,
            rapid,
        }
    }
}

#[derive(Component, Reflect)]
pub struct Movimot {
    // pub motion: MoviMotion,
    pub fast_speed: f32,
    pub slow_speed: f32,
    pub dq: MovimotDQ,
}

impl Movimot {}

#[derive(Bundle)]
pub struct TransportWheelBundle {
    marker: Wheel,
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    collider: Collider,
    motor: Movimot,
}

impl TransportWheelBundle {
    pub fn new(assets: &TBanaAssets, dq: MovimotDQ) -> Self {
        Self {
            marker: Wheel,
            mesh: Mesh3d(assets.wheel_mesh.clone()),
            material: MeshMaterial3d(assets.wheel_materials.ready.clone()),
            collider: assets.wheel_collider.clone(),
            motor: Movimot {
                dq,
                fast_speed: 10.0,
                slow_speed: 2.0,
            },
        }
    }
}

#[derive(Component, Reflect)]
#[relationship(relationship_target = Reciver )]
/// Pushes production details to other
pub struct PushTo(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship=PushTo)]
pub struct Reciver(Vec<Entity>);

#[derive(Component, Reflect)]
#[relationship(relationship_target = Giver )]
pub struct PullFrom(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship=PullFrom)]
pub struct Giver(Vec<Entity>);
