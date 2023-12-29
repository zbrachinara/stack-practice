use bevy::ecs::{system::Query, world::Ref, change_detection::DetectChanges};

use super::{queue::PieceQueue, Active, Hold, Matrix};

pub(super) fn record(state: Query<(Ref<Active>, Ref<PieceQueue>, Ref<Hold>, Ref<Matrix>)>) {
    for (active, queue, hold, matrix) in state.iter() {
        if active.is_changed() || active.is_added() {
            todo!("record change to active piece")
        }

        if queue.is_changed() || queue.is_added() {
            todo!("record change to queue")
        }

        if hold.is_changed() || hold.is_added() {
            todo!("record change to hold slot")
        }
        
        if matrix.is_changed() || matrix.is_added() {
            todo!("record change to matrix")
        }
    }
}
