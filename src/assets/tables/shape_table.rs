use std::fmt::Display;

use bevy::{ecs::system::Resource, math::IVec2, utils::HashMap};

use crate::board::{MinoKind, RotationState};

#[derive(serde::Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[serde(from = "(MinoKind, RotationState)")]
pub struct ShapeParameters {
    pub kind: MinoKind,
    pub rotation: RotationState,
}

impl Display for ShapeParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{:?}", self.kind, self.rotation)
    }
}

impl From<(MinoKind, RotationState)> for ShapeParameters {
    fn from((kind, rotation): (MinoKind, RotationState)) -> Self {
        Self { kind, rotation }
    }
}

#[derive(serde::Deserialize, Resource, Clone, Debug)]
pub struct ShapeTable(pub HashMap<ShapeParameters, Vec<IVec2>>);