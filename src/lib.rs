pub mod attach;
pub mod debug;
pub mod deposit;
pub mod egui;
//pub mod network;
pub mod maps;
pub mod objects;
pub mod physics;
pub mod player;
pub mod traversal;
pub mod prelude;

use bevy_editor_pls::editor::Editor;
use bevy_mod_edge_detection::{EdgeDetectionConfig, EdgeDetectionPlugin};
use bevy_mod_inverse_kinematics::InverseKinematicsPlugin;
use bevy_rapier3d::prelude::*;
use traversal::HierarchyTraversalPlugin;

use self::{
    deposit::DepositPlugin,
    objects::store::StorePlugin,
    physics::{joint_break::BreakJointPlugin, slot::SlotPlugin},
    player::PlayerPlugin,
};

use obj::Obj;

pub use debug::DebugVisible;

//use crate::network::NetworkPlugin;
use bevy::{
    prelude::*,
    scene::SceneInstance,
    window::{CursorGrabMode, WindowPlugin},
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixedSet {
    First,
    Update,
    Last,
}

use bevy_prototype_debug_lines::*;

pub const DEFAULT_FRICTION: Friction = Friction::coefficient(0.5);
pub const TICK_RATE: std::time::Duration = std::time::Duration::from_millis(16);

pub fn setup_app(app: &mut App) {
    //app.insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities);
    let default_res = (1000.0, 600.0);
    //let default_res = (800.0, 500.0);
    //let default_res = (1920.0, 1080.0);
    //let half_width = ((default_res.0 / 2.0), default_res.1);
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Potion Cellar".into(),
                    resolution: default_res.into(),
                    position: WindowPosition::At(IVec2::ZERO),
                    focused: true,
                    present_mode: bevy::window::PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            }),
    );

    app.add_startup_system(
        move |mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>| {
            if let Ok(mut window) = windows.get_single_mut() {
                let center_cursor = Vec2::new(window.width() / 2.0, window.height() / 2.0);
                window.set_cursor_position(Some(center_cursor));
                window.cursor.grab_mode = CursorGrabMode::Locked;
            }
        },
    );

    app.insert_resource(bevy_framepace::FramepaceSettings {
        //limiter: bevy_framepace::Limiter::Off,
        limiter: bevy_framepace::Limiter::Auto,
        //limiter: bevy_framepace::Limiter::Manual(crate::TICK_RATE),
    });
    app.insert_resource(FixedTime::new(crate::TICK_RATE));
    app.add_plugins(bevy_mod_component_mirror::RapierMirrorsPlugins);
    app.add_plugin(bevy_framepace::FramepacePlugin);
    app.insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 2 << 10 });
    app.add_plugin(DebugLinesPlugin::default());
    //app.add_plugin(crate::egui::SetupEguiPlugin);
    app.add_plugin(bevy_editor_pls::EditorPlugin::default());

    app.insert_resource(Msaa::Sample8);
    //app.add_plugin(EdgeDetectionPlugin);
    app.insert_resource(EdgeDetectionConfig {
        debug: 0,
        enabled: 0,
        ..default()
    });

    app.world
        .resource_mut::<Schedules>()
        .get_mut(&CoreSchedule::FixedUpdate)
        .unwrap()
        .configure_sets((FixedSet::First, FixedSet::Update, FixedSet::Last).chain());

    //app.add_plugin(bevy_framepace::FramepacePlugin);
    app.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.3)))
        .add_plugin(PlayerPlugin)
        .add_plugin(attach::AttachPlugin)
        .add_plugin(SlotPlugin)
        .add_plugin(StorePlugin)
        .add_plugin(DepositPlugin)
        .add_plugin(HierarchyTraversalPlugin)
        .add_plugin(BreakJointPlugin)
        .add_plugin(InverseKinematicsPlugin)
        .add_plugin(crate::debug::DebugPlugin)
        //.add_plugin(TreesPlugin)
        .add_plugin(crate::physics::PhysicsPlugin)
        .add_plugin(crate::physics::MusclePlugin)
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: false,
            enabled: true,
            style: Default::default(),
            mode: DebugRenderMode::COLLIDER_SHAPES, //| DebugRenderMode::COLLIDER_AABBS,
        })
        //.add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin);

    //app.add_system(bevy_mod_picking::debug::debug_draw_egui);

    app.add_event::<AssetEvent<Mesh>>();

    app.add_startup_system(fallback_camera);

    app.add_system(update_level_collision);
    app.add_system(active_cameras);
    app.add_system(decomp_load);
    app.add_system(edge_detect_swap);
    //app.add_system(prepare_scene);

    app.add_plugin(crate::player::CustomWanderlustPlugin);
}

fn edge_detect_swap(key: Res<Input<KeyCode>>, mut config: ResMut<EdgeDetectionConfig>) {
    if key.just_pressed(KeyCode::T) {
        config.debug = match config.debug {
            0 => 1,
            _ => 0,
        };
    }
}

fn fallback_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0., 12., 10.))
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            camera: Camera {
                order: -50,
                is_active: false,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Fallback camera"));
}

pub fn active_cameras(_names: Query<&Name>, cameras: Query<(Entity, &Camera)>) {
    let mut active = 0;
    for (_entity, camera) in &cameras {
        if camera.is_active {
            active += 1;
        }
    }

    if active > 1 {
        warn!("More than 1 active camera");
    }
}

#[derive(Component)]
pub struct SpawnedScene;

#[derive(Debug, Component, Clone, Reflect)]
#[reflect(Component)]
pub struct DecompLoad(String);

impl Default for DecompLoad {
    fn default() -> Self {
        Self("".to_owned())
    }
}

fn decomp_load(
    mut commands: Commands,
    mut replace: Query<(Option<&mut Collider>, &DecompLoad, Entity), Changed<DecompLoad>>,
) {
    for (collider, decomp, entity) in &mut replace {
        let path = format!("assets/decomp/obj/{}/obj.obj", decomp.0);
        info!("running decomp load: {:?}", path);
        if let Ok(decomp) = Obj::load(&path) {
            let mut colliders = Vec::new();
            for object in decomp.data.objects {
                let vertices = object
                    .groups
                    .iter()
                    .map(|group| {
                        group
                            .polys
                            .iter()
                            .map(|poly| poly.0.iter().map(|index| index.0))
                    })
                    .flatten()
                    .flatten()
                    .map(|index| decomp.data.position[index])
                    .map(|f| Vec3::from(f))
                    .collect::<Vec<_>>();
                let collider = Collider::convex_hull(&vertices).unwrap();
                colliders.push((Vec3::ZERO, Quat::IDENTITY, collider));
            }

            let new_collider = Collider::compound(colliders);
            match collider {
                Some(mut collider) => {
                    *collider = new_collider;
                }
                None => {
                    commands.entity(entity).insert(new_collider);
                }
            }
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct ColliderLoad;

fn update_level_collision(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Mesh>>,
    mut assets: ResMut<Assets<Mesh>>,
    mut replace: Query<(Option<&mut Collider>, &Handle<Mesh>, Entity), With<ColliderLoad>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                if let Some(loaded_mesh) = assets.get_mut(handle) {
                    for (col, inner_handle, e) in replace.iter_mut() {
                        if *inner_handle == *handle {
                            let new_collider =
                                Collider::from_bevy_mesh(loaded_mesh, &COMPUTE_SHAPE_PARAMS)
                                    .unwrap();
                            match col {
                                Some(mut col) => {
                                    *col = new_collider;
                                }
                                None => {
                                    commands.entity(e).insert(new_collider);
                                }
                            }
                            commands.entity(e).remove::<ColliderLoad>();
                        }
                    }
                }
            }
            AssetEvent::Modified { handle: _ } => {}
            AssetEvent::Removed { handle: _ } => {}
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct SkyLoad;

/*
pub const COMPUTE_SHAPE_PARAMS: ComputedColliderShape = ComputedColliderShape::TriMesh;
*/
pub const COMPUTE_SHAPE_PARAMS: ComputedColliderShape =
    ComputedColliderShape::ConvexDecomposition(VHACDParameters {
        /// Maximum concavity.
        ///
        /// Default: 0.1 (in 2D), 0.01 (in 3D).
        /// Valid range `[0.0, 1.0]`.
        concavity: 0.01,
        /// Controls the bias toward clipping along symmetry planes.
        ///
        /// Default: 0.05.
        /// Valid Range: `[0.0, 1.0]`.
        alpha: 0.05,
        /// Controls the bias toward clipping along revolution planes.
        ///
        /// Default: 0.05.
        /// Valid Range: `[0.0, 1.0]`.
        beta: 0.05,
        /// Resolution used during the voxelization stage.
        ///
        /// Default: 256 (in 2D), 64 (in 3D).
        resolution: 64,
        /// Controls the granularity of the search for the best
        /// clipping plane during the decomposition.
        ///
        /// Default: 4
        plane_downsampling: 4,
        /// Controls the precision of the convex-hull generation
        /// process during the clipping plane selection stage.
        ///
        /// Default: 4
        convex_hull_downsampling: 4,
        /// Controls the way the input mesh or polyline is being
        /// voxelized.
        ///
        /// Default: `FillMode::FloodFill { detect_cavities: false, detect_self_intersections: false }`
        //fill_mode: FillMode::SurfaceOnly,
        fill_mode: FillMode::FloodFill {
            detect_cavities: false,
        },
        /// Controls whether the convex-hull should be approximated during the decomposition stage.
        /// Setting this to `true` increases performances with a slight degradation of the decomposition
        /// quality.
        ///
        /// Default: true
        convex_hull_approximation: true,
        /// Controls the max number of convex-hull generated by the convex decomposition.
        ///
        /// Default: 1024
        max_convex_hulls: 1024,
    });

pub fn mouse_locked(windows: Query<&Window, With<bevy::window::PrimaryWindow>>) -> bool {
    match windows
        .get_single()
        .ok()
        .map(|window| window.cursor.grab_mode == CursorGrabMode::Locked)
    {
        Some(focused) => focused,
        _ => false,
    }
}

pub fn window_focused(windows: Query<&Window, With<bevy::window::PrimaryWindow>>) -> bool {
    match windows.get_single().ok().map(|window| window.focused) {
        Some(focused) => focused,
        _ => false,
    }
}

pub fn editor_active(editor: Option<Res<Editor>>) -> bool {
    if let Some(editor) = editor {
        editor.active()
    } else {
        false
    }
}
