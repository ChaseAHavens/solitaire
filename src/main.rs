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
        .add_plugins(systems::cards::CardsPlugin)
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
            card_start_position: Vec2::ZERO,
            card_draggable: components::cards::CardDraggable { card: None },
            card_id: None,
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
    }
}

#[derive(Resource, Default)]
struct LastClickedEntity(Option<Entity>);

// Really gotta clean up this struct, I think the way I'm using this, card_id and draggable are the
// same entity.
#[derive(Resource, Default)]
struct Dragging {
    draggable: Option<Entity>,
    offset: Vec2,
    card_start_position: Vec2,
    card_id: Option<Entity>,
    card_draggable: components::cards::CardDraggable,
}

fn drag(
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
    let f = pos.0 - last_clicked.offset;
    drag_tx.translation = Vec3::new(f.x, f.y, 0.0);
}

#[derive(Component)]
struct Clickable;

#[derive(Reflect, Clone, Copy, Debug)]
struct Slot {
    position: Vec2,
    slot: components::cards::CardSlot,
}

//will probably just switch to using components for each type of card pile, seems like a better,
//cleaner way to do this then screwing around with whatever I was thinking about here.
#[derive(Resource, Reflect, Clone, Copy, Debug, Default)]
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
    use components::cards::CARD_SIZE;
    let mut pos = CardSlotPositions::new();
    let card_width = CARD_SIZE.x;
    let card_spacing = Vec2::new(card_width + PADDING, 0.0);
    let board_width = (card_width * 7.0) + (PADDING * 6.0);
    let p = Vec2::new(board_width / -2.0, 0.0);
    let s = build_slot_to_spawn(p);
    pos.stock_pile = Some(Slot {
        position: p,
        slot: s.1,
    });
    commands.spawn(s);
    let p = p + card_spacing;
    let s = build_slot_to_spawn(p);
    pos.waste_pile = Some(Slot {
        position: p,
        slot: s.1,
    });
    commands.spawn(s);
    let p = p + card_spacing;
    let p = p + card_spacing;
    for i in 0..4 {
        let p = p + (card_spacing * i as f32);
        let s = build_slot_to_spawn(p);
        pos.foundations[i] = Some(Slot {
            position: p,
            slot: s.1,
        });
        commands.spawn(s);
    }
    //second row
    let p = Vec2::new(board_width / -2.0, -(CARD_SIZE.y + PADDING));
    for i in 0..7 {
        let p = p + (card_spacing * i as f32);
        let s = build_slot_to_spawn(p);
        pos.tableau[i] = Some(Slot {
            position: p,
            slot: s.1,
        });
        commands.spawn(s);
    }
    commands.insert_resource(pos);
}

fn build_slot_to_spawn(pos: Vec2) -> (SpatialBundle, components::cards::CardSlot) {
    let slot = components::cards::CardSlot;
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
                );
            }
            MoveState::Moving => {
                moving_stuff(tx, all_draggables, c, current_time);
            }
            MoveState::EndMove => {
                c.moving = MoveState::RemoveComponent;
            }
            MoveState::RemoveComponent => {
                println!("Removing MoveThisCard component");
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
    let time_since_start_of_move = (c.time_to_finish_move - c.time_at_start_of_move) as f32;
    let percent = (t - c.time_at_start_of_move) as f32 / time_since_start_of_move;
    let percent_of_move_done: f32 = if percent >= 1.0 { 1.0 } else { percent };
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
        println!("Setting move state to end");
        c.moving = MoveState::EndMove;
    }
}

fn start_new_move(tx: &mut Transform, e: Option<Entity>, c: &mut MoveThisCard, t: u128) {
    println!("current time is : {}", t);
    println!("time to finish is : {}", c.time_to_finish_move);
    c.target = e;
    //c.time_at_start_of_move = t;
    //c.time_to_finish_move = t + r.gen_range(2000..3000);
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

fn mouse_input(
    mut commands: Commands,
    mouse_clicks: Res<Input<MouseButton>>,
    mut drag: ResMut<Dragging>,
    pos: Res<MousePosition>,
    time: Res<Time>,
    draggables: Query<(Entity, &Transform, &components::cards::CardDraggable)>,
) {
    if mouse_clicks.just_released(MouseButton::Left) {
        if drag.card_id.is_none() {
            return;
        }
        let the_card_id = drag
            .card_id
            .expect("None card_id when trying to build MoveThisCard");
        let the_card_draggable = drag
            .card_draggable
            .card
            .expect("None from card draggable when trying to build MoveThisCard");
        commands.entity(the_card_draggable).insert(MoveThisCard {
            target: Some(the_card_id),
            start_position: drag.card_start_position,
            moving: MoveState::StartMove,
            time_at_start_of_move: time.elapsed().as_millis(),
            time_to_finish_move: time.elapsed().as_millis() + 300,
            rotation_freqs: (0, 1, 0),
        });
        drag.draggable = None;
    }
    if mouse_clicks.just_pressed(MouseButton::Left) {
        use components::cards;
        let size = cards::CARD_SIZE;
        //I can tell I'm messing up with the distance mesaure, its not always grabbing the center
        //of the cards, this might be the upper left corner, but this needs to all be replaced
        //anyway and just grab cards by the highest z card that you click on.
        let mut distance = 0.0;
        let mut selected: Option<(Entity, &Transform, &cards::CardDraggable)> = None;
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
            //println!("Card selected: {:?}", selected);
            dbg!(selected);
            drag.draggable = Some(x);
            let current_card_pos = tx.translation.truncate();
            drag.offset = pos.0 - current_card_pos;
            drag.card_start_position = current_card_pos;
            drag.card_id = Some(x);
            drag.card_draggable = *cd;
        }
        println!("Mouse clicked at {}, {}", pos.0.x, pos.0.y);
    }
}

fn keyboard_input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    test: Res<CardSlotPositions>,
    current: Res<components::cards::CurrentCard>,
    mut cards: Query<(&mut Transform, &mut components::cards::CardVisual)>,
    gizmos_toggle: Res<inspector::GizmosDraw>,
) {
    let the_cards = &mut cards;
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
        println!("{:#?}", temp2);
        temp2.translation = temp2.translation + temp2.local_x() * 25.0;
    }
    if keys.just_pressed(KeyCode::G) {
        println!("{:#?}", test);
    }
    if keys.just_pressed(KeyCode::D) {
        commands.insert_resource(inspector::GizmosDraw(!gizmos_toggle.0));
    }
    commands.insert_resource(components::cards::CurrentCard(index));
}

fn setup(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    let rng = &mut rand::thread_rng();
    for i in 0..52 {
        use components::cards::{self, CardColor, CardSuit, CardVisual, CARD_SIZE};
        let texture_handle = asset_server.load("cards.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            CARD_SIZE,
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
                suit = CardSuit::Hearts;
                color = CardColor::Red;
            }
            (13..=25) => {
                suit = CardSuit::Spades;
                color = CardColor::Black;
            }
            (26..=39) => {
                suit = CardSuit::Diamonds;
                color = CardColor::Red;
            }
            (40..=52) => {
                suit = CardSuit::Clubs;
                color = CardColor::Black;
            }
            _ => {
                suit = CardSuit::Hearts;
                color = CardColor::Black;
                println!("got out of range when generating cards")
            }
        }
        let c = CardVisual {
            index: i + 1,
            number: (i % 13) + 1,
            suit,
            color,
        };
        let initial_position = Vec3::new(
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
                    cards::CardFront,
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
                    cards::CardBack,
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
                inspector::DebugRect,
                cards::CardDraggable { card: Some(ent) },
                Clickable,
            ))
            .id();
        commands.entity(ent).insert(MoveThisCard {
            target: Some(card_drag),
            start_position: initial_position.truncate(),
            moving: MoveState::StartMove,
            time_at_start_of_move: time.elapsed().as_millis(),
            time_to_finish_move: time.elapsed().as_millis() + 1000,
            rotation_freqs: (rng.gen_range(-1..=1), rng.gen_range(-1..=1), 0),
        });
    }
}
