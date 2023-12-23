use self::shape_table::ShapeParameters;

pub mod kick_table;
pub mod shape_table;
pub mod sprite_table;

/// Returns all possible shape parameters
fn all_shape_parameters() -> impl Iterator<Item = ShapeParameters> {
    use crate::board::MinoKind::*;
    [T, O, L, J, S, Z, I]
        .into_iter()
        .flat_map(|kind| {
            use crate::board::RotationState::*;
            [(kind, Up), (kind, Left), (kind, Down), (kind, Right)]
        })
        .map(ShapeParameters::from)
}

#[cfg(test)]
mod test {
    use super::all_shape_parameters;

    #[test]
    fn correct_shape_parameters() {
        assert_eq!(all_shape_parameters().count(), 28)
    }
}
