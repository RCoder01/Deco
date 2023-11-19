use std::{collections::HashMap, f32::consts::PI};

use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    asset::LoadedFolder,
    core_pipeline::clear_color::ClearColorConfig,
    ecs::system::EntityCommands,
    gltf::Gltf,
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::{vec4, DVec2},
    pbr::NotShadowCaster,
    prelude::*,
    render::primitives::Aabb,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_flycam::{FlyCam, KeyBindings, MovementSettings, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{backends::raycast::bevy_mod_raycast::prelude::SimplifiedMesh, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NoCameraPlayerPlugin)
        // .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_state::<LoadingState>()
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 12.0,          // default: 12.0
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::E,
            move_descend: KeyCode::Q,
            toggle_grab_cursor: KeyCode::R,
            ..Default::default()
        })
        .init_resource::<LoadedModelList>()
        .init_resource::<AabbMeshMap>()
        .add_event::<ModelMoveEvent>()
        .add_systems(
            Startup,
            (model_loader, spawn_inital_scene, spawn_ui, set_title),
        )
        .add_systems(
            Update,
            (
                mouse_scroll,
                dropdown_system,
                button_system,
                recenter_mouse,
                make_pickable,
                move_model,
            ),
        )
        .add_systems(
            Update,
            check_asset_loading.run_if(in_state(LoadingState::Unloaded)),
        )
        .add_systems(
            Update,
            gltf_asset_event_watcher, //.run_if(in_state(LoadingState::Loaded)),
        )
        .run();
}

fn set_title(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = window.get_single_mut() else {
        error!("Unable to get primary window");
        return;
    };
    window.title = "DECO.ai".to_string();
}

fn spawn_inital_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    const W: f32 = 40.;
    const L: f32 = 50.;
    const H: f32 = 20.;
    let skybox_handle: Handle<Image> = asset_server.load("images/Ryfjallet_cubemap.png");
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10., 2. * H, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.3, 0.6, 0.85)),
                ..Default::default()
            },
            ..Default::default()
        },
        // Skybox(skybox_handle.clone()),
        FlyCam,
    ));
    Box::leak(Box::new(skybox_handle));
    // Floor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(L, W)).into()),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/wood.png")),
                perceptual_roughness: 0.8,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_xyz(0., 0., 0.).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -PI / 2.,
                0.,
                PI / 2.,
            )),
            ..default()
        },
        Pickable::IGNORE,
        NotShadowCaster,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(L, H)).into()),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/wallpaper.png")),
                perceptual_roughness: 0.8,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_xyz(W / 2., H / 2., 0.).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -PI / 2.,
                -PI / 2.,
                PI / 2.,
            )),
            ..default()
        },
        Pickable::IGNORE,
        NotShadowCaster,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(W, H)).into()),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/wallpaper.png")),
                perceptual_roughness: 0.8,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_xyz(0., H / 2., L / 2.).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.,
                0.,
                0.,
            )),
            ..default()
        },
        Pickable::IGNORE,
        NotShadowCaster,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(L, H)).into()),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/wallpaper.png")),
                perceptual_roughness: 0.8,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_xyz(-W / 2., H / 2., 0.).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -PI / 2.,
                -PI / 2.,
                PI / 2.,
            )),
            ..default()
        },
        Pickable::IGNORE,
        NotShadowCaster,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(W, H)).into()),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/wallpaper.png")),
                perceptual_roughness: 0.8,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_xyz(0., H / 2., -L / 2.).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.,
                0.,
                0.,
            )),
            ..default()
        },
        Pickable::IGNORE,
        NotShadowCaster,
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 1.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            PI / 2. - 1.,
            1.0,
            -PI / 4.,
        )),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
    // commands.spawn(PointLightBundle {
    //     transform: Transform::from_translation(Vec3::new(0., 50., 0.)),
    //     point_light: PointLight {
    //         intensity: 8000.0,
    //         ..default()
    //     },
    //     ..default()
    // });
}

#[derive(Debug, States, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum LoadingState {
    #[default]
    Unloaded,
    Loaded,
}

#[derive(Debug, Resource)]
struct AssetFolder(Handle<LoadedFolder>);

#[derive(Debug, Resource, Default)]
struct LoadedModelList(Vec<Handle<Gltf>>);

#[derive(Component)]
struct ModelListParent;

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

#[derive(Component)]
struct BooleanComponent(bool);

#[derive(Component)]
struct ListItemModel(Handle<Gltf>);

#[derive(Resource, Default)]
struct AabbMeshMap(HashMap<Handle<Mesh>, Handle<Mesh>>);

#[derive(Event)]
struct ModelMoveEvent(Entity, Drag);

impl From<ListenerInput<Pointer<Drag>>> for ModelMoveEvent {
    fn from(value: ListenerInput<Pointer<Drag>>) -> Self {
        Self(value.listener(), (**value).clone())
    }
}

fn model_loader(mut commands: Commands, asset_server: Res<AssetServer>) {
    let folder = asset_server.load_folder("models");
    commands.insert_resource(AssetFolder(folder));
}

const RIGHT_SIDEBAR_WIDTH: f32 = 250.;

fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Bold.ttf");
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            // left vertical fill (border)
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(0.),
                    border: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                ..default()
            });
            // right vertical fill
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Start,
                        width: Val::Px(RIGHT_SIDEBAR_WIDTH),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Title
                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(RIGHT_SIDEBAR_WIDTH),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_sections([
                                    TextSection::new(
                                        "Available models",
                                        TextStyle {
                                            font_size: 25.,
                                            ..default()
                                        },
                                    ),
                                    TextSection::new(
                                        " ▼",
                                        TextStyle {
                                            font,
                                            font_size: 25.,
                                            ..default()
                                        },
                                    ),
                                ]),
                                Label,
                                BooleanComponent(true),
                            ));
                        });
                    // List with hidden overflow
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                align_self: AlignSelf::Stretch,
                                // height: Val::Percent(90.),
                                overflow: Overflow::clip_y(),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            // Moving panel
                            parent.spawn((
                                NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ..default()
                                },
                                ScrollingList::default(),
                                AccessibilityNode(NodeBuilder::new(Role::List)),
                                ModelListParent,
                            ));
                        });
                });
        });
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn recenter_mouse(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut last_mouse_pos: Local<Option<Vec2>>,
) {
    let mut window = primary_window.get_single_mut().unwrap();
    if window.cursor.grab_mode == CursorGrabMode::Confined && last_mouse_pos.is_none() {
        *last_mouse_pos = window.cursor_position();
    }
    if window.cursor.grab_mode != CursorGrabMode::Confined {
        *last_mouse_pos = None;
        return;
    }
    if let Some(last_pos) = *last_mouse_pos {
        window.set_physical_cursor_position(Some(DVec2::new(last_pos.x as f64, last_pos.y as f64)));
    }
}

fn dropdown_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<(&mut Text, &mut BooleanComponent)>,
    mut model_list_parent: Query<
        &mut Visibility,
        (With<ModelListParent>, Without<Button>, Without<Text>),
    >,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let Ok((mut text, mut bool_component)) = text_query.get_mut(children[0]) else {
            continue;
        };
        match *interaction {
            Interaction::Pressed => {
                let Ok(mut model_list_parent_visibility) = model_list_parent.get_single_mut()
                else {
                    continue;
                };
                bool_component.0 = !bool_component.0;
                if bool_component.0 {
                    text.sections[1].value = " ▼".to_string();
                    *model_list_parent_visibility = Visibility::Inherited;
                } else {
                    text.sections[1].value = " ▲".to_string();
                    *model_list_parent_visibility = Visibility::Hidden;
                }
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut model_query: Query<&ListItemModel>,
    gltf_assets: Res<Assets<Gltf>>,
    camera_pos: Query<&GlobalTransform, With<Camera3d>>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let Ok(model) = model_query.get_mut(children[0]) else {
            continue;
        };
        match *interaction {
            Interaction::Pressed => {
                let Ok(camera_pos) = camera_pos.get_single() else {
                    error!("Unable to find camera transform while spawning model");
                    continue;
                };
                let mut transform = camera_pos.compute_transform();
                let looking = transform.rotation * Vec3::Z;
                transform.translation -= transform.translation.y * looking * 1.5;
                transform.translation.y = 0.;
                let rot_y = transform.rotation.to_euler(EulerRot::XYZ).1;
                transform.rotation = Quat::from_euler(EulerRot::XYZ, 0., rot_y, 0.);
                let mesh = gltf_assets
                    .get(&model.0)
                    .expect(&format!("Expected to find asset",));
                let scene = mesh
                    .default_scene
                    .as_ref()
                    .or(mesh.scenes.first())
                    .expect("Expected model to have at least one scene")
                    .clone();
                commands.spawn((
                    SceneBundle {
                        scene,
                        transform,
                        ..default()
                    },
                    On::<Pointer<Drag>>::send_event::<ModelMoveEvent>(),
                ));
                // text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                // border_color.0 = Color::RED;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                // border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                // text.sections[0].value = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                // border_color.0 = Color::BLACK;
            }
        }
    }
}

fn move_model(
    mut models: Query<
        &mut Transform,
        (
            With<Handle<Scene>>,
            With<On<Pointer<Drag>>>,
            Without<Camera3d>,
        ),
    >,
    camera: Query<&Transform, With<Camera3d>>,
    mut move_events: EventReader<ModelMoveEvent>,
) {
    let Ok(camera) = camera.get_single() else {
        error!("No camera!?");
        return;
    };
    let camera_rotation = camera.rotation.to_euler(EulerRot::XYZ).2;
    let (sin, cos) = camera_rotation.sin_cos();
    for event in move_events.read() {
        let Ok(mut model) = models.get_mut(event.0) else {
            error!("Event for nonexistent model");
            continue;
        };
        let delta = event.1.delta;
        match event.1.button {
            PointerButton::Primary => {
                model.translation.x += (delta.x * cos + delta.y * sin) * 0.05;
                model.translation.z += (-delta.x * sin + delta.y * cos) * 0.03;
            }
            PointerButton::Secondary => {
                model.rotate_local_y(event.1.delta.x / 50.0);
                model.scale *= (event.1.delta.y / -100.).exp().min(10.);
            }
            PointerButton::Middle => {
                model.translation.y += event.1.delta.y * -0.05;
            }
        }
    }
}

fn check_asset_loading(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<AssetFolder>,
) {
    let loading = loading.0.clone();
    match server.load_state(loading) {
        bevy::asset::LoadState::Loaded => {
            println!("All assets loaded!");
            commands.insert_resource(NextState(Some(LoadingState::Loaded)))
        }
        bevy::asset::LoadState::Failed => panic!("Unable to load all assets"),
        _ => {}
    }
}

fn gltf_asset_event_watcher(
    mut commands: Commands,
    mut gltf_events: EventReader<AssetEvent<Gltf>>,
    folder_resource: Res<AssetFolder>,
    mut gltf_resource: ResMut<LoadedModelList>,
    folder_asset: Res<Assets<LoadedFolder>>,
    model_list_parent: Query<Entity, With<ModelListParent>>,
    mut event_queue: Local<Vec<AssetEvent<Gltf>>>,
) {
    let Some(loaded_folder): Option<&LoadedFolder> = folder_asset.get(&folder_resource.0) else {
        event_queue.extend(gltf_events.read());
        return;
    };
    let size_before = gltf_resource.0.len();
    let Ok(parent) = model_list_parent.get_single() else {
        return;
    };
    for gevent in event_queue.iter().chain(gltf_events.read()) {
        dbg!(gevent);
        match gevent {
            AssetEvent::Added { id } => {
                let Some(gltf_asset) = loaded_folder
                    .handles
                    .iter()
                    .find(|handle| handle.id() == id.untyped())
                else {
                    continue;
                };
                gltf_resource.0.push(gltf_asset.clone().typed());
                spawn_model(commands.entity(parent), gltf_asset.clone().typed());
            }
            _ => {}
        }
    }
    event_queue.clear();
    if gltf_resource.0.len() != size_before {
        dbg!(gltf_resource);
    }
}

fn spawn_model(mut entity_commands: EntityCommands, gltf_asset: Handle<Gltf>) {
    entity_commands.with_children(|parent| {
        parent
            .spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(RIGHT_SIDEBAR_WIDTH),
                    // height: Val::Px(65.0),
                    // border: UiRect::all(Val::Px(5.0)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        match gltf_asset.path() {
                            Some(path) => {
                                let path = path
                                    .path()
                                    .file_stem()
                                    .expect("Path should have a file name")
                                    .to_owned()
                                    .into_string()
                                    .expect("Path should be valid UTF-8");
                                path
                            }
                            None => "Unknown".to_string(),
                        },
                        TextStyle {
                            font_size: 20.,
                            ..default()
                        },
                    ),
                    Label,
                    AccessibilityNode(NodeBuilder::new(Role::ListItem)),
                    ListItemModel(gltf_asset),
                ));
            });
    });
}

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;

            let max_scroll = (items_height - container_height).max(0.);

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            style.top = Val::Px(scrolling_list.position);
        }
    }
}

fn make_pickable(
    mut commands: Commands,
    meshes: Query<(Entity, &Aabb, &Handle<Mesh>), Without<Pickable>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut aabb_mesh_map: ResMut<AabbMeshMap>,
) {
    for (entity, aabb, mesh) in meshes.iter() {
        let aabb_box = shape::Box::from_corners(
            (aabb.center - aabb.half_extents).into(),
            (aabb.center + aabb.half_extents).into(),
        );
        let aabb_handle = aabb_mesh_map
            .0
            .entry(mesh.clone())
            .or_insert_with(|| mesh_assets.add(aabb_box.into()))
            .clone();
        commands.entity(entity).insert((
            PickableBundle::default(),
            HIGHLIGHT_TINT.clone(),
            SimplifiedMesh { mesh: aabb_handle },
        ));
    }
}

const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.5, -0.3, 0.9, 0.8), // hovered is blue
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, -0.4, 0.8, 0.8), // pressed is a different blue
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, 0.8, -0.4, 0.0), // selected is green
        ..matl.to_owned()
    })),
};
