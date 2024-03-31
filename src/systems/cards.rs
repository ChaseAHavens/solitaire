use bevy::prelude::*;

pub struct CardsPlugin;

impl Plugin for CardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (card_visual_keep_face_up, card_visual_keep_back_up));
    }
}

pub fn card_visual_keep_face_up(
    mut cards: Query<(
        &mut Visibility,
        &mut GlobalTransform,
        &crate::components::cards::CardFront,
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
pub fn card_visual_keep_back_up(
    mut cards: Query<(
        &mut Visibility,
        &mut GlobalTransform,
        &crate::components::cards::CardBack,
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
