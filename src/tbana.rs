use std::borrow::Cow;

use bevy::color::palettes::css;
use bevy::prelude::*;

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
        app.init_resource::<TBanaAssets>();
        app.add_systems(Startup, load_assets.in_set(InitSet::LoadAssets));
    }
}

pub fn load_assets(
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut tbana_res: ResMut<TBanaAssets>,
) {
    let block_mesh = mesh_assets.add(Extrusion::new(Rectangle::default(), 2.0));
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

    tbana_res.base_mesh = block_mesh;
    tbana_res.base_materials.ready = ready_mode;
    tbana_res.base_materials.running = run_mode;
    tbana_res.base_materials.alarm = alarm_mode;
}

#[derive(Default)]
struct BaseMaterials {
    ready: Handle<StandardMaterial>,
    running: Handle<StandardMaterial>,
    alarm: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct TBanaAssets {
    base_mesh: Handle<Mesh>,
    base_materials: BaseMaterials,
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
            mesh: Mesh3d(tbana_assets.base_mesh.clone()),
            material: MeshMaterial3d(tbana_assets.base_materials.ready.clone()),
        }
    }
}

#[derive(Component)]
struct TransportBana;

#[derive(Reflect, Component, Default, Deref)]
struct AutoMode {
    enabled: bool,
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
