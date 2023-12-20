use bevy::{ecs::system::Resource, math::IVec2, utils::HashMap};

use crate::board::{MinoKind, RotationState};

#[derive(serde::Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[serde(from = "(MinoKind, RotationState)")]
struct ShapeParameters {
    kind: MinoKind,
    rotation: RotationState,
}

impl From<(MinoKind, RotationState)> for ShapeParameters {
    fn from((kind, rotation): (MinoKind, RotationState)) -> Self {
        Self { kind, rotation }
    }
}

#[derive(serde::Deserialize, Resource, Clone, Debug)]
pub struct ShapeTable(HashMap<ShapeParameters, Vec<IVec2>>);
