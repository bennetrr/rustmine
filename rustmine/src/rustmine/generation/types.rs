use crate::rustmine::generation::chunk::{Chunk, ChunkMesh};
use cgmath::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Constants
pub const CHUNK_SIZE: i32 = 16; // Should not be hardcoded but whatever

// Types
pub(crate) type BlockPos = Vector3<i32>; // Block Position in world coordinates
pub(crate) type ChunkPos = Vector2<i32>;
pub(crate) type PlayerPos = Vector3<f32>; // Player Position in world coordinates
pub type CameraData = ([f32; 3], [f32; 3], [f32; 3], bool);

// Enums

/// A finished unit of work produced by the [`ChunkGenerator`] worker pool.
///
/// The terrain path returns the freshly generated [`Chunk`] (it is the authoritative
/// block data). The decoration/remesh path returns *only* the position and mesh — it
/// deliberately does not carry blocks back, so a stale block snapshot taken when the
/// remesh was queued can never overwrite the authoritative chunk in the world.
pub(crate) enum ChunkResult {
    Generated(Chunk, HashMap<BlockPos, BlockType>),
    Decorated(ChunkPos, ChunkMesh),
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Copy, Hash, Eq, PartialOrd, Ord)]
pub(crate) enum ChunkStatus {
    Generated,
    Decorated,
}

/// BlockType enum.
///
/// To add a new block you must:<br>
/// In the `rustmine/src/assets/textures`: Add an atlas, mtl and obj files,<br>
/// In `types.rs`: Add the type in the BlockType enum,<br>
/// In `chunk.rs`: In `generate_mesh()` add vector of instances, push the instances in the match statement, return the instances.<br>
/// In `game_state.rs`: Add a block model in the "Models Insertion" code block.
/// Add a chunk block group in the "Chunk Groups Insertion" code block.
/// Add instances in the "Add instances to the chunk_groups" code block.<br>
/// In `render()` of `game_state.rs`: Add ChunkBlockGroup
/// in the "Procedural ChunkBlockGroups Insertion" code block.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash)]
pub(crate) enum BlockType {
    // Natural Blocks
    Grass,
    Dirt,
    Cobblestone,
    Stone,

    // Wood
    Oak,
    Spruce,
    Birch,

    // Leaves
    LeavesOak,
    LeavesBirch,
    LeavesSpruce,

    // Other blocks
    TallGrass,
}
