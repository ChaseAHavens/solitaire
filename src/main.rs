use bevy::{prelude::KeyCode, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
//use bevy_window::PrimaryWindow;
mod components;
mod inspector;
//mod components/cards;
//use crate::components::cards;
use rand::prelude::IteratorRandom;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(inspector::InspectorPlugin)
        .register_type::<components::cards::CardSuit>()
        .register_type::<components::cards::CardColor>()
        .register_type::<components::cards::Card>()
        .register_type::<MoveCardsWtihDelay>()
        .register_type::<u128>()
        .insert_resource(components::cards::Cards { cards: Vec::new() })
        .insert_resource(components::cards::CurrentCard(0))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                inspector::gizmo_update,
                move_cards_with_delay,
                //_test_system,
                keyboard_input,
                card_face_up,
                card_back_up,
                //_spin_spinnners,
                //inspector_ui.run_if(input_toggle_active(true, KeyCode::Escape)),
            ),
        )
        .run();
}
/*
fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
    let mut egui_context = world
    .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
    .single(world)
    .clone();
    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Hierarchy");

                bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
                    world,
                    ui,
                    &mut selected_entities,
                );

                ui.label("Press escape to toggle UI");
                ui.allocate_space(ui.available_size());
            });
        });

        egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Inspector");

                match selected_entities.as_slice() {
                    &[entity] => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                    }
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world, entities, ui,
                        );
                    }
                }

                ui.allocate_space(ui.available_size());
            });
        });
    }
    */
/*
#[derive(Component)]
struct Spinner {
    x: f32,
    y: f32,
    z: f32,
//}
*/

#[derive(Reflect, Clone, Copy, Debug)]
enum MoveState {
    StartMove,
    Moving,
    EndMove,
    Waiting,
}

#[derive(Component, Reflect, Clone, Copy, Debug)]
pub struct MoveCardsWtihDelay {
    target: Option<Entity>,
    start_position: Vec2,
    moving: MoveState,
    time_at_start_of_move: u128,
    time_before_next_move: u128,
    time_to_finish_move: u128,
    rotation_freqs: (i8, i8, i8),
}

fn move_cards_with_delay(
    time: Res<Time>,
    mut move_card: Query<(&mut Transform, &mut MoveCardsWtihDelay)>,
    slots: Query<(Entity, &Transform, &components::cards::CardSlot), Without<MoveCardsWtihDelay>>,
) {
    let rng = &mut rand::thread_rng();
    let current_time = time.elapsed().as_millis();
    for (mut txa, mut ca) in &mut move_card.iter_mut() {
        let tx = txa.as_mut();
        let c = ca.as_mut();
        let stx: Vec<(Entity, &Transform)> = slots.iter().map(|x| (x.0, x.1)).collect();

        let random_slot: Option<Entity> = Some(slots.iter().choose(rng).unwrap().0);
        match c.moving {
            MoveState::StartMove => {
                start_new_move(tx, random_slot, c, current_time, rng);
            }
            MoveState::Moving => {
                moving_stuff(tx, stx, c, current_time);
            }
            MoveState::EndMove => {
                c.time_before_next_move = current_time + rng.gen_range(1000..2000);
                c.moving = MoveState::Waiting;
            }
            MoveState::Waiting => {
                if c.time_before_next_move >= current_time {
                    c.moving = MoveState::StartMove;
                }
            }
        }
    }
    fn moving_stuff(
        tx: &mut Transform,
        slots: Vec<(Entity, &Transform)>,
        c: &mut MoveCardsWtihDelay,
        t: u128,
    ) {
        let slot_position_of_current_card = slots
            .iter()
            .find(|x| x.0 == c.target.unwrap())
            .unwrap()
            .1
            .translation;
        let percent_of_move_done: f32 = (t - c.time_at_start_of_move) as f32
            / (c.time_to_finish_move - c.time_at_start_of_move) as f32;
        let location: Vec2 = c.start_position.lerp(
            slot_position_of_current_card.truncate(),
            percent_of_move_done,
        );
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
        c: &mut MoveCardsWtihDelay,
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
}

fn card_face_up(
    mut cards: Query<(
        &mut Visibility,
        &mut GlobalTransform,
        &components::cards::CardFront,
    )>,
) {
    for (mut vis, tx, _) in cards.iter_mut() {
        let dot = (tx.back()).dot(Vec3::Z);
        *vis = if dot > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
fn card_back_up(
    mut cards: Query<(
        &mut Visibility,
        &mut GlobalTransform,
        &components::cards::CardBack,
    )>,
) {
    for (mut vis, tx, _) in cards.iter_mut() {
        let dot = (tx.back()).dot(Vec3::Z);
        *vis = if dot < 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
/*
fn _spin_spinnners(time: Res<Time>, mut cards: Query<(&mut Transform, &Spinner, &Card)>) {
    for (mut tx, s, _) in cards.iter_mut() {
        tx.rotate_x(s.x * time.delta_seconds());
        tx.rotate_y(s.y * time.delta_seconds());
        tx.rotate_z(s.z * time.delta_seconds());
    //}
//}
*/

fn _test_system(
    time: Res<Time>,
    mut cards: Query<
        (&mut Transform, &components::cards::Card),
        Without<components::cards::CardSlot>,
    >,
    slots: Query<(&Transform, &components::cards::CardSlot), Without<components::cards::Card>>,
) {
    let first_slot = slots.iter().next().expect("fuck").0;
    for (mut t, _) in cards.iter_mut() {
        let diff = t.translation - first_slot.translation;
        let dir = diff.normalize();
        t.translation -= dir * 50.0 * time.delta_seconds();
    }
}

fn keyboard_input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    current: Res<components::cards::CurrentCard>,
    mut cards: Query<(&mut Transform, &mut components::cards::Card)>,
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
    if keys.just_pressed(KeyCode::Left) {
        // Left Ctrl was released
    }
    if keys.pressed(KeyCode::W) {
        // W is being held down
    }
    // we can check multiple at once with `.any_*`
    if keys.any_pressed([KeyCode::M, KeyCode::N]) {
        // Either the left or right shift are being held down
    }
    if keys.any_just_pressed([KeyCode::Delete, KeyCode::Back]) {
        // Either delete or backspace was just pressed
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
        let c: components::cards::Card = components::cards::Card {
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
        commands
            .spawn((
                SpatialBundle {
                    transform: Transform {
                        translation: initial_position,
                        ..default()
                    },
                    ..default()
                },
                c,
                /*
                Spinner {
                    x: rng.gen_range(-3.0..=3.0),
                    y: rng.gen_range(-3.0..=3.0),
                    z: rng.gen_range(-3.0..=3.0),
                //},
                */
                MoveCardsWtihDelay {
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
            });
    }
}
