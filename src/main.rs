use bevy::{prelude::KeyCode, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
mod components;
mod inspector;
mod systems;
use bevy_window::PrimaryWindow;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(inspector::InspectorPlugin)
        .add_plugins(crate::systems::cards::CardsPlugin)
        .register_type::<components::cards::CardSuit>()
        .register_type::<components::cards::CardColor>()
        .register_type::<components::cards::CardVisual>()
        .register_type::<MoveThisCard>()
        .register_type::<u128>()
        .register_type::<CardSlotPositions>()
        .insert_resource(components::cards::Cards { cards: Vec::new() })
        .insert_resource(LastClickedEntity(None))
        .insert_resource(Dragging {
            draggable: None,
            offset: Vec2::ZERO,
        })
        .insert_resource(components::cards::CurrentCard(0))
        .init_resource::<MousePosition>()
        .add_systems(Startup, (setup, generate_board))
        .add_systems(
            Update,
            (
                move_cards,
                keyboard_input,
                mouse_input,
                mouse_position_system,
                drag,
                // This is probably not a good way to do this, will have to reasearch more about
                // better ways to handel input later
                //click_check_system.run_if(bevy::input::common_conditions::input_just_pressed(MouseButton::Left, )),
            ),
        )
        .run();
}

#[derive(Resource, Default)]
struct MousePosition(Vec2);

fn mouse_position_system(
    mut pos: ResMut<MousePosition>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (cam, cam_tx) = camera.single();
    let win = window.single();
    if let Some(world_pos) = win
        .cursor_position()
        .and_then(|c| cam.viewport_to_world(cam_tx, c))
        .map(|ray| ray.origin.truncate())
    {
        pos.0 = world_pos;
        //println!("pos is {}, {}", world_pos.x, world_pos.y);
    }
}

#[derive(Resource, Default)]
struct LastClickedEntity(Option<Entity>);

#[derive(Resource, Default)]
struct Dragging {
    draggable: Option<Entity>,
    offset: Vec2,
}

fn drag(
    //mut commands: Commands,
    pos: Res<MousePosition>,
    last_clicked: Res<Dragging>,
    mut draggables: Query<(Entity, &mut Transform, &components::cards::CardDraggable)>,
) {
    if last_clicked.draggable.is_none() {
        return;
    }
    let (_drag_ent, mut drag_tx, _drag_able) = draggables
        .iter_mut()
        .find(|x| Some(x.0) == last_clicked.draggable)
        .expect("Draggable saved in Dragging doesnt match any CardDraggables queried.");
    let f = pos.0 + last_clicked.offset;
    drag_tx.translation = Vec3::new(f.x, f.y, 0.0);
}

#[derive(Component)]
struct Clickable;

//The start of this is all fixed now so the visual cards move towards the
//draggable cards, I still have to fix this so it stops crashing and
//write it so you can click and drag the draggables and then the visual
//cards just move at the end of the click and drag.

#[derive(Reflect, Clone, Copy, Debug)]
struct Slot {
    position: Vec2,
    slot: components::cards::CardSlot,
}

#[derive(Reflect, Clone, Copy, Debug)]
struct CardSlotPositions {
    stock_pile: Option<Slot>,
    waste_pile: Option<Slot>,
    foundations: [Option<Slot>; 4],
    tableau: [Option<Slot>; 7],
}

impl CardSlotPositions {
    fn new() -> CardSlotPositions {
        CardSlotPositions {
            stock_pile: None,
            waste_pile: None,
            foundations: [None, None, None, None],
            tableau: [None, None, None, None, None, None, None],
        }
    }
}

//const BOARD_POSITION_OFFSET: Vec2 = Vec2::new(-400.0, 310.0);
const PADDING: f32 = 7.0;

fn generate_board(mut commands: Commands) {
    let mut pos: CardSlotPositions = CardSlotPositions::new();
    let card_width = crate::components::cards::CARD_SIZE.x;
    let card_spacing = Vec2::new(card_width + PADDING, 0.0);
    let board_width = (card_width * 7.0) + (PADDING * 6.0);
    let p = Vec2::new(board_width / -2.0, 0.0);
    let s = spawn_slot(p);
    pos.stock_pile = Some(Slot {
        position: p,
        slot: s.1,
    });
    commands.spawn(s);
    let p = p + card_spacing;
    let s = spawn_slot(p);
    pos.waste_pile = Some(Slot {
        position: p,
        slot: s.1,
    });
    commands.spawn(s);
    let p = p + card_spacing;
    let p = p + card_spacing;
    for i in 0..4 {
        let p = p + (card_spacing * i as f32);
        let s = spawn_slot(p);
        pos.foundations[i] = Some(Slot {
            position: p,
            slot: s.1,
        });
        commands.spawn(s);
    }
    //second row
    let p = Vec2::new(
        board_width / -2.0,
        -(crate::components::cards::CARD_SIZE.y + PADDING),
    );
    for i in 0..7 {
        let p = p + (card_spacing * i as f32);
        let s = spawn_slot(p);
        pos.tableau[i] = Some(Slot {
            position: p,
            slot: s.1,
        });
        commands.spawn(s);
    }
}

fn spawn_slot(pos: Vec2) -> (SpatialBundle, components::cards::CardSlot) {
    let slot: components::cards::CardSlot = components::cards::CardSlot;
    (
        bevy::prelude::SpatialBundle {
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, 0.0),
                ..default()
            },
            ..default()
        },
        slot,
    )
}

#[derive(Reflect, Clone, Copy, Debug)]
enum MoveState {
    StartMove,
    Moving,
    EndMove,
    RemoveComponent,
}

#[derive(Component, Reflect, Clone, Copy, Debug)]
pub struct MoveThisCard {
    target: Option<Entity>,
    start_position: Vec2,
    moving: MoveState,
    time_at_start_of_move: u128,
    time_before_next_move: u128,
    time_to_finish_move: u128,
    rotation_freqs: (i8, i8, i8),
}

fn move_cards(
    mut commands: Commands,
    time: Res<Time>,
    mut move_card: Query<(Entity, &mut Transform, &mut MoveThisCard)>,
    draggables: Query<
        (Entity, &Transform, &components::cards::CardDraggable),
        Without<MoveThisCard>,
    >,
) {
    let rng = &mut rand::thread_rng();
    let current_time = time.elapsed().as_millis();
    for (ea, mut txa, mut ca) in &mut move_card.iter_mut() {
        let tx = txa.as_mut();
        let c = ca.as_mut();
        let all_draggables: Vec<(Entity, &Transform)> =
            draggables.iter().map(|x| (x.0, x.1)).collect();

        let slot_from_movethiscard = draggables
            .iter()
            .find(|x| x.0 == c.target.expect("fuckin in slot_from"));
        if slot_from_movethiscard.is_none() {
            println!("No CardDraggable found matching this MoveThisCard target.");
            return;
        }
        match c.moving {
            MoveState::StartMove => {
                start_new_move(
                    tx,
                    Some(
                        slot_from_movethiscard
                            .expect("Somehow got past the early return on none")
                            .0,
                    ),
                    c,
                    current_time,
                    rng,
                );
            }
            MoveState::Moving => {
                moving_stuff(tx, all_draggables, c, current_time);
            }
            MoveState::EndMove => {
                c.time_before_next_move = current_time + rng.gen_range(1000..2000);
                c.moving = MoveState::RemoveComponent;
            }
            MoveState::RemoveComponent => {
                commands.entity(ea).remove::<MoveThisCard>();
            }
        }
    }
}

fn moving_stuff(
    tx: &mut Transform,
    targets: Vec<(Entity, &Transform)>,
    c: &mut MoveThisCard,
    t: u128,
) {
    let current_move_target = targets
        .iter()
        .find(|x| {
            x.0 == c
                .target
                .expect("Got a none while trying to unwrap in Moving_stuff")
        })
        .expect("Got a none trying to unwrap an (&entity, &transform) in moving_stuff")
        .1
        .translation;
    let percent_of_move_done: f32 = (t - c.time_at_start_of_move) as f32
        / (c.time_to_finish_move - c.time_at_start_of_move) as f32;
    let location: Vec2 = c
        .start_position
        .lerp(current_move_target.truncate(), percent_of_move_done);
    tx.translation = location.extend(tx.translation.z);
    let x = (c.rotation_freqs.0 as f32 * (percent_of_move_done * 360.0)) % 360.0;
    let y = (c.rotation_freqs.1 as f32 * (percent_of_move_done * 360.0)) % 360.0;
    let z = (c.rotation_freqs.2 as f32 * (percent_of_move_done * 360.0)) % 360.0;
    tx.rotation = Quat::from_euler(
        EulerRot::XYZ,
        x.to_radians(),
        y.to_radians(),
        z.to_radians(),
    );
    if c.time_to_finish_move <= t {
        c.moving = MoveState::EndMove;
    }
}

fn start_new_move(
    tx: &mut Transform,
    e: Option<Entity>,
    c: &mut MoveThisCard,
    t: u128,
    r: &mut rand::rngs::ThreadRng,
) {
    c.target = e;
    c.time_at_start_of_move = t;
    c.time_before_next_move = t + r.gen_range(1000..2000);
    c.time_to_finish_move = t + r.gen_range(2000..3000);
    c.start_position = tx.translation.truncate();
    c.moving = MoveState::Moving;
}

fn _test_system(
    time: Res<Time>,
    mut cards: Query<
        (&mut Transform, &components::cards::CardVisual),
        Without<components::cards::CardSlot>,
    >,
    slots: Query<
        (&Transform, &components::cards::CardSlot),
        Without<components::cards::CardVisual>,
    >,
) {
    let first_slot = slots.iter().next().expect("fuck").0;
    for (mut t, _) in cards.iter_mut() {
        let diff = t.translation - first_slot.translation;
        let dir = diff.normalize();
        t.translation -= dir * 50.0 * time.delta_seconds();
    }
}

//this needs to be updated and all the clicking code moved into this so I can get rid of that
//weird run_if thing in the setup.
fn mouse_input(
    mut commands: Commands,
    mouse_clicks: Res<Input<MouseButton>>,
    mut drag: ResMut<Dragging>,
    pos: Res<MousePosition>,
    draggables: Query<(Entity, &Transform, &components::cards::CardDraggable)>,
) {
    if mouse_clicks.just_released(MouseButton::Left) {
        drag.draggable = None;
        /*
        commands.insert_resource(Dragging {
            draggable: None,
            offset: Vec2::ZERO,
        });
        */
    }
    if mouse_clicks.just_pressed(MouseButton::Left) {
        let size = crate::components::cards::CARD_SIZE;
        let mut distance = 0.0;
        let mut selected: Option<(Entity, &Transform, &components::cards::CardDraggable)> = None;
        for d in draggables.iter() {
            let card_rect = Rect::from_center_size(d.1.translation.truncate(), size);
            if !card_rect.contains(pos.0) {
                continue;
            }
            let this_card_distance = pos.0.distance(d.1.translation.truncate());
            if this_card_distance < distance {
                continue;
            }
            distance = this_card_distance;
            selected = Some(d);
        }
        if let Some((x, tx, cd)) = selected {
            println!("Card selected: {:?}", selected);
            drag.draggable = Some(x);
            let current_card_pos = tx.translation.truncate();
            commands.entity(x).insert(MoveThisCard {
                target: Some(cd.card),
                start_position: current_card_pos,
                moving: MoveState::StartMove,
                time_at_start_of_move: 0,
                time_before_next_move: 0,
                time_to_finish_move: 0,
                rotation_freqs: (0, 1, 0),
            });
        }
        println!("Mouse clicked at {}, {}", pos.0.x, pos.0.y);
    }
}
/*
fn click_check_system(
    mut commands: Commands,
    pos: Res<MousePosition>,
    mut selected_entity: ResMut<LastClickedEntity>,
    clk: Query<(Entity, &Transform, &components::cards::CardDraggable)>,
) {
    let size = crate::components::cards::CARD_SIZE;
    let mut distance = 0.0;
    let mut selected: Option<(Entity, &Transform, &components::cards::CardDraggable)> = None;
    for c in clk.iter() {
        let card_rect = Rect::from_center_size(c.1.translation.truncate(), size);
        if !card_rect.contains(pos.0) {
            continue;
        }
        let this_card_distance = pos.0.distance(c.1.translation.truncate());
        if this_card_distance < distance {
            continue;
        }
        distance = this_card_distance;
        selected = Some(c);
    }
    if let Some((x, tx, cd)) = selected {
        println!("Card selected: {:?}", selected);
        selected_entity.0 = Some(x);
        let current_card_pos = tx.translation.truncate();
        commands.entity(x).insert(MoveThisCard {
            target: Some(cd.card),
            start_position: current_card_pos,
            moving: MoveState::StartMove,
            time_at_start_of_move: 0,
            time_before_next_move: 0,
            time_to_finish_move: 0,
            rotation_freqs: (0, 1, 0),
        });
    }
    println!("Mouse clicked at {}, {}", pos.0.x, pos.0.y);
}
*/
fn keyboard_input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    current: Res<components::cards::CurrentCard>,
    mut cards: Query<(&mut Transform, &mut components::cards::CardVisual)>,
    gizmos_toggle: Res<inspector::GizmosDraw>,
) {
    let the_cards = &mut cards; //.iter();
    let it = &mut the_cards.iter_mut();
    let mut index: usize = current.0;
    if keys.just_pressed(KeyCode::Space) {
        println!("{:?}", it.nth(index));
    }
    if keys.just_pressed(KeyCode::Up) {
        print!("before up {} , ", index);
        index += 1;
        println!("after up {}", index);
    }
    if keys.just_pressed(KeyCode::Down) {
        print!("before up {} , ", index);
        index -= 1;
        println!("after up {}", index);
    }
    if keys.just_pressed(KeyCode::Right) {
        let t = it.nth(index);
        let temp = t.expect("shit");
        let (mut temp2, _) = temp;
        println!("{:?}", temp2);
        temp2.translation = temp2.translation + temp2.local_x() * 25.0;
    }
    if keys.just_pressed(KeyCode::D) {
        commands.insert_resource(inspector::GizmosDraw(!gizmos_toggle.0));
    }
    commands.insert_resource(components::cards::CurrentCard(index));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    let rng = &mut rand::thread_rng();
    /*
        for _ in 0..10 {
            commands.spawn((
                SpatialBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            rng.gen_range(-300.0..=300.0),
                            rng.gen_range(-300.0..=300.0),
                            0.0,
                        ),
                        ..default()
                    },
                    ..default()
                },
                components::cards::CardSlot,
            ));
        }
    */
    for i in 0..52 {
        let texture_handle = asset_server.load("cards.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            components::cards::CARD_SIZE,
            13,
            4,
            Some(Vec2::new(3.0, 3.0)),
            None,
        );
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        let suit;
        let color;
        match i {
            (0..=12) => {
                suit = components::cards::CardSuit::Hearts;
                color = components::cards::CardColor::Red;
            }
            (13..=25) => {
                suit = components::cards::CardSuit::Spades;
                color = components::cards::CardColor::Black;
            }
            (26..=39) => {
                suit = components::cards::CardSuit::Diamonds;
                color = components::cards::CardColor::Red;
            }
            (40..=52) => {
                suit = components::cards::CardSuit::Clubs;
                color = components::cards::CardColor::Black;
            }
            _ => {
                suit = components::cards::CardSuit::Hearts;
                color = components::cards::CardColor::Black;
                println!("got out of range when generating cards")
            }
        }
        let c: components::cards::CardVisual = components::cards::CardVisual {
            index: i + 1,
            number: (i % 13) + 1,
            suit,
            color,
        };
        let initial_position: Vec3 = Vec3::new(
            rng.gen_range(-300.0..=300.0),
            rng.gen_range(-300.0..=300.0),
            rng.gen_range(-300.0..=300.0),
        );

        let ent = commands
            .spawn((
                SpatialBundle {
                    transform: Transform {
                        translation: Vec3::ZERO, //initial_position,
                        ..default()
                    },
                    ..default()
                },
                c,
                //Clickable,
                /*
                MoveThisCard {
                    target: None,
                    start_position: initial_position.truncate(),
                    moving: MoveState::StartMove,
                    time_at_start_of_move: 0,
                    time_before_next_move: 0,
                    time_to_finish_move: 0,
                    rotation_freqs: (
                        rng.gen_range(-1..=1),
                        rng.gen_range(-1..=1),
                        0, //rng.gen_range(0..=1),
                    ),
                },
                */
            ))
            .with_children(|parent| {
                parent.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle,
                        sprite: TextureAtlasSprite {
                            index: i,
                            ..default()
                        },
                        ..default()
                    },
                    components::cards::CardFront,
                ));
            })
            .with_children(|parent| {
                parent.spawn((
                    SpriteBundle {
                        texture: asset_server.load("back.png"),
                        transform: Transform {
                            scale: Vec3::new(0.78, 0.72, 1.0),
                            ..default()
                        },

                        ..default()
                    },
                    components::cards::CardBack,
                ));
            })
            .id();
        let card_drag = commands
            .spawn((
                SpatialBundle {
                    transform: Transform {
                        translation: initial_position,
                        ..default()
                    },
                    ..default()
                },
                crate::inspector::DebugRect,
                crate::components::cards::CardDraggable { card: ent },
                Clickable,
                /*
                MoveThisCard {
                    target: None,
                    start_position: initial_position.truncate(),
                    moving: MoveState::StartMove,
                    time_at_start_of_move: 0,
                    time_before_next_move: 0,
                    time_to_finish_move: 0,
                    rotation_freqs: (
                        rng.gen_range(-1..=1),
                        rng.gen_range(-1..=1),
                        0, //rng.gen_range(0..=1),
                    ),
                },
                */
            ))
            .id();
        commands.entity(ent).insert(MoveThisCard {
            target: Some(card_drag),
            start_position: initial_position.truncate(),
            moving: MoveState::StartMove,
            time_at_start_of_move: 0,
            time_before_next_move: 0,
            time_to_finish_move: 0,
            rotation_freqs: (
                rng.gen_range(-1..=1),
                rng.gen_range(-1..=1),
                0, //rng.gen_range(0..=1),
            ),
        });
    }
}
