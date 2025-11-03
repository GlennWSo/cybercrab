use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bitvec::BitArr;

use crate::{physics::PhysLayer, tbana::TransportBana, InitSet};

pub struct ShiftRegPlugin;

impl Plugin for ShiftRegPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RegisterPosition>();
        app.register_type::<Register>();
        app.init_resource::<DetailAssets>();
        app.insert_resource(Register::new(50));
        app.add_systems(Startup, load_assets.in_set(InitSet::LoadAssets));
        app.add_systems(Startup, spawn_test_detail.in_set(InitSet::Spawn));
        app.add_systems(Update, animate_test_detail);
        app.add_observer(on_shift_over);
    }
}

fn spawn_test_detail(mut cmd: Commands, assets: Res<DetailAssets>) {
    let bundle = (
        DetailBundle::new(&assets),
        Name::new("Detail_1"),
        RigidBody::Kinematic,
        Transform::from_xyz(0.0, 0.6, 0.0),
        CollidingEntities::default(),
        LinearVelocity(Vec3 {
            z: 0.1,
            ..Default::default()
        }),
    );
    cmd.spawn(bundle);
}

fn animate_test_detail(
    details: Query<&mut Transform, With<Detail>>,
    tbanor: Query<&Transform, (With<TransportBana>, Without<Detail>)>,
) {
    let max_z = tbanor
        .into_iter()
        .map(|trans| trans.translation.z)
        .reduce(|acc, v| acc.max(v))
        .unwrap()
        + 2.0;
    for mut transform in details {
        if transform.translation.z > max_z {
            transform.translation.z -= 20.0;
        }
    }
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
    marker: Detail,
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    collider: Collider,
    phys_layers: CollisionLayers,
}

impl DetailBundle {
    pub fn new(assets: &DetailAssets) -> Self {
        Self {
            marker: Detail,
            mesh: Mesh3d(assets.base_shape.clone()),
            material: MeshMaterial3d(assets.normal_material.clone()),
            collider: assets.collider.clone(),
            phys_layers: CollisionLayers::new(PhysLayer::Detail, LayerMask::ALL),
        }
    }
}

#[derive(Resource, Default)]
pub struct DetailAssets {
    base_shape: Handle<Mesh>,
    normal_material: Handle<StandardMaterial>,
    collider: Collider, // TODO turn into Asset/Handle
}

pub type RegisterIndex = u16;

#[derive(Component, Reflect, Deref, DerefMut, Clone, Copy, Debug)]
pub struct RegisterPosition(pub RegisterIndex);

impl RegisterPosition {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Default, Clone, Reflect, Debug)]
pub struct DetailState {
    #[reflect(ignore)]
    state_bits: BitArr!(for 32, in u8),
    #[reflect(ignore)]
    bits_set: BitArr!(for 32, in u8),
}
impl DetailState {
    pub fn get_bit(&self, idx: usize) -> Option<bool> {
        if !self.bits_set[idx] {
            return None;
        }
        Some(self.state_bits[idx])
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct Register {
    pub details: Vec<Option<DetailState>>,
}

#[derive(Event)]
pub struct ShiftOver {
    pub from: Entity,
    pub to: Entity,
}

fn on_shift_over(trigger: On<ShiftOver>, q: Query<&RegisterPosition>, mut reg: ResMut<Register>) {
    let Ok([from, to]) = q.get_many([trigger.from, trigger.to]) else {
        return;
    };
    let Some(detail) = reg.details[from.as_usize()].take() else {
        return;
    };
    let to_state = &mut reg.details[to.as_usize()];
    if to_state.is_some() {
        panic!("cant shift part to occuipeid pos");
        // return;
    }
    *to_state = Some(detail);
}

impl Register {
    pub fn new(n_details: usize) -> Self {
        let mut details = vec![None; n_details];
        if n_details > 0 {
            details[0] = Some(DetailState::default());
        }
        Self { details }
    }
    pub fn pop_detail(&mut self) -> Option<DetailState> {
        if self.details.len() == 0 {
            return None;
        }
        let idx = self.details.len() - 1;
        self.details[idx].take()
    }
    pub fn shift_detail_forward(&mut self, idx: usize) -> Result<()> {
        if self.details.len() == idx {
            let detail = self.details[idx].take();
            self.details.push(detail);
            return Ok(());
        } else if self.details.len() < idx {
            return Err(format!("target idx {idx} is to big").into());
        }

        let detail = self.details[idx].take();
        let Some(next) = self.details.get_mut(idx + 1) else {
            return Err(format!("Next slot already taken {idx}").into());
        };
        *next = detail;
        Ok(())
    }
}
