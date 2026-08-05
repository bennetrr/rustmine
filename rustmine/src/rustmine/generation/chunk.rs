use crate::engine::rendering::instance::Instance;
use crate::rustmine::generation::tree::{generate_tree, hash_pos};
use crate::rustmine::generation::types::{BlockPos, ChunkPos, ChunkResult};
use crate::rustmine::generation::types::{BlockType, ChunkStatus};
use crate::rustmine::generation::world::GenerationData;
use cgmath::{One, Vector2, Vector3};
use crossbeam::channel::{Receiver, Sender, unbounded};
use noise::{NoiseFn, SuperSimplex};
use rand::random_bool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

pub(crate) struct ChunkMesh {
    pub pos: ChunkPos,
    pub grass: Vec<Instance>,
    pub dirt: Vec<Instance>,
    pub cobblestone: Vec<Instance>,
    pub stone: Vec<Instance>,

    // Wood
    pub oak: Vec<Instance>,
    pub birch: Vec<Instance>,
    pub spruce: Vec<Instance>,

    // Leaves
    pub leaves_oak: Vec<Instance>,
    pub leaves_birch: Vec<Instance>,
    pub leaves_spruce: Vec<Instance>,

    // Other blocks
    pub tall_grass: Vec<Instance>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Chunk {
    pub(crate) blocks: HashMap<BlockPos, BlockType>,
    pub(crate) position: ChunkPos,
    pub(crate) status: ChunkStatus,
    /// Indicates that the chunk was changed and needs to be recalculated at next render
    pub(crate) is_dirty: bool,
}

impl Chunk {
    /// Constructs a new chunk at `position`, loading generation parameters from JSON
    /// and procedurally generating its blocks using `seed`.
    pub fn new(data: &GenerationData, position: ChunkPos) -> Self {
        let blocks = Self::generate_blocks(data, position);
        Self {
            blocks,
            position,
            status: ChunkStatus::Generated,
            is_dirty: true,
        }
    }

    /// Generates all blocks for a chunk using SuperSimplex noise.
    ///w
    /// Each XZ column gets a noise-derived surface height, placing:
    /// - `Grass` at the surface
    /// - `Dirt` for `dirt_depth` layers below the surface
    /// - `Cobblestone` from `cobble_min_y` up to the dirt layer
    fn generate_blocks(data: &GenerationData, position: ChunkPos) -> HashMap<BlockPos, BlockType> {
        let mut blocks = HashMap::new();
        let chunk_size = data.chunk_size as i32;
        let super_simplex = SuperSimplex::new(data.seed);

        for x in 0..chunk_size {
            for z in 0..chunk_size {
                let pos_x = x + chunk_size * position.x;
                let pos_z = z + chunk_size * position.y;

                let biome =
                    generate_octaves(&super_simplex, pos_x as f64, pos_z as f64, 1, 0.0005, 1.0);
                let mountain_factor = ((biome + 1.0) / 2.0).powf(2.0);

                let height =
                    generate_octaves(&super_simplex, pos_x as f64, pos_z as f64, 4, 0.002, 1.0);

                let amplitude = 4.0 + mountain_factor * 120.0;
                let base_y = 16.0 + mountain_factor * 20.0;

                let surface_y = (height * amplitude + base_y) as i32;
                for y in data.cobblestone_min_y..=surface_y {
                    let block_type = if y == surface_y {
                        BlockType::Grass
                    } else if y >= surface_y - data.dirt_depth as i32 {
                        BlockType::Dirt
                    } else {
                        BlockType::Stone
                    };

                    blocks.insert(
                        Vector3 {
                            x: pos_x,
                            y,
                            z: pos_z,
                        },
                        block_type,
                    );
                }
                if random_bool(1f64 / 5f64) {
                    blocks.insert(
                        Vector3 {
                            x: pos_x,
                            y: surface_y + 1,
                            z: pos_z,
                        },
                        BlockType::TallGrass,
                    );
                }
            }
        }

        blocks
    }

    pub fn get_block(&self, pos: BlockPos) -> Option<&BlockType> {
        self.blocks.get(&pos)
    }

    pub fn get_y(&self, pos: Vector2<i32>) -> i32 {
        self.blocks
            .keys()
            .filter_map(|position| {
                if pos.x == position.x && pos.y == position.z {
                    Some(position.y)
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(i32::MIN)
    }

    pub fn set_block(&mut self, pos: BlockPos, block_type: BlockType) {
        self.mark_as_dirty();
        self.blocks.insert(pos, block_type);
    }

    pub fn remove_block(&mut self, pos: BlockPos) -> Option<BlockType> {
        self.mark_as_dirty();
        self.blocks.remove(&pos)
    }

    /// Marks the chunk to be recalculated at next render
    pub fn mark_as_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn mark_as_clean(&mut self) {
        self.is_dirty = false;
    }
}

pub(crate) struct ChunkGenerator {
    pub request_tx: Sender<ChunkPos>,
    pub result_rx: Receiver<ChunkResult>,
    pub remesh_tx: Sender<(ChunkPos, HashMap<BlockPos, BlockType>)>,
}

impl ChunkGenerator {
    /// Creates a new [`ChunkGenerator`] with the specified number of worker threads.
    ///
    /// Spawns `worker_count` threads, each of which listens for chunk positions on a shared
    /// channel. When a position is received, the worker generates the chunk's blocks and mesh,
    /// then sends the result back on the result channel.
    ///
    /// # Arguments
    ///
    /// * `worker_count` - Number of worker threads to spawn.
    ///
    /// * `data` - Shared world-generation parameters (noise seeds, biome config, etc.)
    ///   wrapped in an [`Arc`] so each worker gets an inexpensive clone.
    ///
    /// # Panics
    ///
    /// Does not panic. Channel errors inside workers are silently ignored via `.ok()`.
    pub(crate) fn new(worker_count: usize, data: Arc<GenerationData>) -> Self {
        let (request_tx, request_rx) = unbounded::<ChunkPos>();
        let (result_tx, result_rx) = unbounded::<ChunkResult>();
        let (remesh_tx, remesh_rx) = unbounded::<(ChunkPos, HashMap<BlockPos, BlockType>)>();

        // Terrain workers
        for _ in 0..worker_count {
            let rx = request_rx.clone();
            let tx = result_tx.clone();
            let data = Arc::clone(&data);
            thread::spawn(move || {
                while let Ok(pos) = rx.recv() {
                    let blocks = Chunk::generate_blocks(&data, pos);

                    // Precompute this chunk's trees here too. A tree's root is always a
                    // grass block belonging to the chunk that spawned it, so this needs
                    // no neighbour data - safe to do off-thread. This used to run on the
                    // main thread in `World::collect_generated_chunks`, scanning every
                    // block in the chunk synchronously every frame.
                    let mut tree_blocks = HashMap::new();
                    for (block_pos, block_type) in &blocks {
                        if *block_type == BlockType::Grass
                            && hash_pos(block_pos.x, block_pos.z, data.seed).is_multiple_of(137)
                        {
                            generate_tree(
                                &mut tree_blocks,
                                block_pos.x,
                                block_pos.y,
                                block_pos.z,
                                data.seed,
                            );
                        }
                    }

                    let chunk = Chunk {
                        blocks,
                        position: pos,
                        status: ChunkStatus::Generated,
                        is_dirty: true,
                    };
                    tx.send(ChunkResult::Generated(chunk, tree_blocks)).ok();
                }
            });
        }

        // Decorative workers
        for _ in 0..worker_count {
            let rx = remesh_rx.clone();
            let tx = result_tx.clone();
            thread::spawn(move || {
                while let Ok((pos, blocks)) = rx.recv() {
                    let mesh = generate_mesh(&blocks, pos, [None, None, None, None]);
                    tx.send(ChunkResult::Decorated(pos, mesh)).ok();
                }
            });
        }

        Self {
            request_tx,
            result_rx,
            remesh_tx,
        }
    }

    /// Queues a chunk position for asynchronous generation.
    ///
    /// Sends `pos` to the worker pool over an unbounded channel. The call returns
    /// immediately; the actual generation happens on a background thread. Results
    /// can be retrieved later with [`ChunkGenerator::poll_ready`].
    ///
    /// Sending only fails if all worker threads have panicked and dropped their
    /// receiver ends, in which case the error is silently discarded.
    pub fn request(&self, pos: ChunkPos) {
        let _ = self.request_tx.send(pos);
    }

    /// Drains all chunk results that have finished generating since the last call.
    ///
    /// Non-blocking: returns immediately with whatever is available in the result
    /// channel. An empty [`Vec`] means no chunks have finished yet, not that none
    /// were requested.
    ///
    /// Intended to be called once per frame from the main thread. The returned
    /// pairs should be inserted into the world and uploaded to the GPU before the
    /// next render.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of [`ChunkResult`] values — one entry per completed unit of work,
    /// in completion order.
    pub fn poll_ready(&self) -> Vec<ChunkResult> {
        let mut ready = Vec::new();
        while let Ok(item) = self.result_rx.try_recv() {
            ready.push(item);
        }
        ready
    }
}

/// Generates instances (mesh) for a specific chunk.
/// Culls specific blocks, which can't be seen by a player.
pub(crate) fn generate_mesh(
    blocks: &HashMap<BlockPos, BlockType>,
    pos: ChunkPos,
    neighbours: [Option<&HashMap<BlockPos, BlockType>>; 4],
) -> ChunkMesh {
    let mut grass = Vec::new();
    let mut dirt = Vec::new();
    let mut cobblestone = Vec::new();
    let mut stone = Vec::new();
    let mut oak = Vec::new();
    let mut spruce = Vec::new();
    let mut birch = Vec::new();
    let mut leaves_oak = Vec::new();
    let mut leaves_birch = Vec::new();
    let mut leaves_spruce = Vec::new();
    let mut tall_grass = Vec::new();

    let chunk_min_x = pos.x * 16;
    let chunk_min_z = pos.y * 16;
    let is_air = |x: i32, y: i32, z: i32| -> bool {
        let out_x = x < chunk_min_x || x >= chunk_min_x + 16;
        let out_z = z < chunk_min_z || z >= chunk_min_z + 16;

        if out_x || out_z {
            let neighbour = if x >= chunk_min_x + 16 {
                neighbours[0]
            } else if x < chunk_min_x {
                neighbours[1]
            } else if z >= chunk_min_z + 16 {
                neighbours[2]
            } else {
                neighbours[3]
            };

            return match neighbour {
                Some(n) => !n.contains_key(&Vector3::new(x, y, z)),
                None => false,
            };
        }

        !blocks.contains_key(&Vector3::new(x, y, z))
    };

    for (block_pos, block_type) in blocks {
        let visible = is_air(block_pos.x + 1, block_pos.y, block_pos.z)
            || is_air(block_pos.x - 1, block_pos.y, block_pos.z)
            || is_air(block_pos.x, block_pos.y + 1, block_pos.z)
            || is_air(block_pos.x, block_pos.y - 1, block_pos.z)
            || is_air(block_pos.x, block_pos.y, block_pos.z + 1)
            || is_air(block_pos.x, block_pos.y, block_pos.z - 1);

        // Add block_type != BlockType if the culling of the block does not make sense. (i.e. ore or leaves)
        if !visible
            && *block_type != BlockType::Grass
            && *block_type != BlockType::LeavesOak
            && *block_type != BlockType::LeavesBirch
            && *block_type != BlockType::LeavesSpruce
            && *block_type != BlockType::Oak
            && *block_type != BlockType::Birch
            && *block_type != BlockType::Spruce
        {
            continue;
        }

        let instance = Instance {
            position: Vector3 {
                x: block_pos.x as f32,
                y: block_pos.y as f32,
                z: block_pos.z as f32,
            },
            rotation: cgmath::Quaternion::one(),
        };

        match block_type {
            BlockType::Grass => grass.push(instance),
            BlockType::Dirt => dirt.push(instance),
            BlockType::Cobblestone => cobblestone.push(instance),
            BlockType::Stone => stone.push(instance),

            // Wood
            BlockType::Oak => oak.push(instance),
            BlockType::Spruce => spruce.push(instance),
            BlockType::Birch => birch.push(instance),

            // Leaves
            BlockType::LeavesOak => leaves_oak.push(instance),
            BlockType::LeavesBirch => leaves_birch.push(instance),
            BlockType::LeavesSpruce => leaves_spruce.push(instance),

            // Other Blocks
            BlockType::TallGrass => tall_grass.push(instance),
        }
    }

    ChunkMesh {
        pos,
        grass,
        dirt,
        cobblestone,
        stone,

        // Wood
        oak,
        spruce,
        birch,

        // Leaves
        leaves_oak,
        leaves_birch,
        leaves_spruce,

        // Other blocks
        tall_grass,
    }
}

/// Samples layered SuperSimplex noise at `(x, z)` with the given number of octaves.
///
/// Each octave doubles the frequency and halves the amplitude. The result is
/// normalized by the sum of all amplitudes, keeping the output in `[-1.0, 1.0]`.
fn generate_octaves(
    super_simplex: &SuperSimplex,
    x: f64,
    z: f64,
    octaves: u32,
    base_freq: f64,
    base_amp: f64,
) -> f64 {
    let mut total = 0.0;
    let mut frequency = base_freq;
    let mut amplitude = base_amp;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += super_simplex.get([x * frequency, z * frequency]) * amplitude;
        max_value += amplitude;
        frequency *= 2.0;
        amplitude *= 0.5;
    }

    total / max_value
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Vector2;

    fn create_test_data() -> GenerationData {
        GenerationData::new(12345, 16, 3, -64)
    }

    fn create_test_chunk() -> Chunk {
        let position = Vector2 { x: 0, y: 0 };
        Chunk::new(&create_test_data(), position)
    }

    #[test]
    fn test_chunk_creation() {
        let chunk = create_test_chunk();

        assert!(!chunk.blocks.is_empty(), "Chunk sollte Blöcke enthalten");
        assert_eq!(chunk.position.x, 0, "Position X sollte 0 sein");
        assert_eq!(chunk.position.y, 0, "Position Y sollte 0 sein");
    }

    #[test]
    fn test_get_block_exists() {
        let chunk = create_test_chunk();

        // Awaited Positions (0, surface_y, 0) should exist
        let surface_y = chunk.get_y(Vector2 { x: 0, y: 0 });
        let pos = Vector3 {
            x: 0,
            y: surface_y,
            z: 0,
        };

        let block = chunk.get_block(pos);
        assert!(
            block.is_some(),
            "Block an Position {:?} sollte existieren",
            pos
        );

        if let Some(block_type) = block {
            assert!(
                matches!(block_type, BlockType::Grass),
                "Oberster Block sollte Gras sein"
            );
        }
    }

    #[test]
    fn test_get_block_not_exists() {
        let chunk = create_test_chunk();

        // Position outside the generated world
        let pos = Vector3 {
            x: 9999,
            y: 9999,
            z: 9999,
        };
        let block = chunk.get_block(pos);

        assert!(
            block.is_none(),
            "Block an Position {:?} sollte nicht existieren",
            pos
        );
    }

    #[test]
    fn test_set_block() {
        let mut chunk = create_test_chunk();
        let pos = Vector3 {
            x: 100,
            y: 100,
            z: 100,
        };

        // Set block
        chunk.set_block(pos, BlockType::Oak);

        // Check whether block exists
        let block = chunk.get_block(pos);
        assert!(block.is_some(), "Block sollte nach set_block existieren");

        if let Some(block_type) = block {
            assert_eq!(*block_type, BlockType::Oak, "Block sollte Oak sein");
        }
    }

    #[test]
    fn test_set_block_overwrite() {
        let mut chunk = create_test_chunk();
        let pos = Vector3 {
            x: 100,
            y: 100,
            z: 100,
        };

        // Set first block
        chunk.set_block(pos, BlockType::Grass);

        // Overwrite Block
        chunk.set_block(pos, BlockType::Oak);

        let block = chunk.get_block(pos).unwrap();
        assert_eq!(
            *block,
            BlockType::Oak,
            "Block sollte mit Oak überschrieben sein"
        );
    }

    #[test]
    fn test_remove_block() {
        let mut chunk = create_test_chunk();
        let pos = Vector3 {
            x: 100,
            y: 100,
            z: 100,
        };

        chunk.set_block(pos, BlockType::Grass);
        assert!(chunk.get_block(pos).is_some(), "Block sollte existieren");

        // Remove Block
        let removed = chunk.remove_block(pos);
        assert!(removed.is_some(), "remove_block should return Some");
        assert_eq!(
            removed.unwrap(),
            BlockType::Grass,
            "Removed block should be Grass"
        );

        // Check if block was removed
        assert!(
            chunk.get_block(pos).is_none(),
            "Block should not exist after remove"
        );
    }

    #[test]
    fn test_remove_nonexistent_block() {
        let mut chunk = create_test_chunk();
        let pos = Vector3 {
            x: 9999,
            y: 9999,
            z: 9999,
        };

        let removed = chunk.remove_block(pos);
        assert!(
            removed.is_none(),
            "remove_block on a non-existent block should return None"
        );
    }

    #[test]
    fn test_get_y_returns_highest_block() {
        let mut chunk = create_test_chunk();

        // Setting multiple blocks at the same X,Z position
        let pos = Vector2 { x: 50, y: 50 };

        chunk.set_block(
            Vector3 {
                x: 50,
                y: 10,
                z: 50,
            },
            BlockType::Dirt,
        );
        chunk.set_block(
            Vector3 {
                x: 50,
                y: 20,
                z: 50,
            },
            BlockType::Dirt,
        );
        chunk.set_block(
            Vector3 {
                x: 50,
                y: 30,
                z: 50,
            },
            BlockType::Grass,
        );

        let highest_y = chunk.get_y(pos);
        assert_eq!(highest_y, 30, "highest block must be Y=30");
    }

    #[test]
    fn test_get_y_returns_lowest_when_no_blocks() {
        let chunk = create_test_chunk();

        // Position, die keine Blöcke hat
        let pos = Vector2 { x: -1000, y: -1000 };
        let y = chunk.get_y(pos);

        // if no blocks, the minimal value should be returned
        assert_eq!(y, i32::MIN, "Without blocks i32::MIN should be returned");
    }

    #[test]
    fn test_generate_blocks_has_grass_on_surface() {
        let chunk = create_test_chunk();

        // Check all grass blocks
        let grass_blocks: Vec<_> = chunk
            .blocks
            .iter()
            .filter(|(_, block_type)| matches!(block_type, BlockType::Grass))
            .collect();

        assert!(
            !grass_blocks.is_empty(),
            "There should be at least one grass block"
        );

        // Check if grass blocks are on the surface
        for (pos, _) in grass_blocks {
            let above = Vector3 {
                x: pos.x,
                y: pos.y + 1,
                z: pos.z,
            };
            let above_block = chunk.get_block(above);

            // Above grass there should be either nothing or no grass
            if let Some(block) = above_block {
                assert!(
                    !matches!(block, BlockType::Grass),
                    "Above grass there should be either nothing or no grass"
                );
            }
        }
    }

    #[test]
    fn test_generate_blocks_has_dirt_below_grass() {
        let chunk = create_test_chunk();

        // Finde einen Gras-Block
        let grass_pos = chunk
            .blocks
            .iter()
            .find(|(_, block_type)| matches!(block_type, BlockType::Grass))
            .map(|(pos, _)| *pos);

        if let Some(grass_pos) = grass_pos {
            // Below the grass there should be dirt
            let below = Vector3 {
                x: grass_pos.x,
                y: grass_pos.y - 1,
                z: grass_pos.z,
            };
            let below_block = chunk.get_block(below);

            assert!(
                below_block.is_some(),
                "Below the grass there should be a block"
            );
            if let Some(block) = below_block {
                assert!(
                    matches!(block, BlockType::Dirt),
                    "Below the grass there should be dirt"
                );
            }
        }
    }

    #[test]
    fn test_generate_octaves_works() {
        let simplex = SuperSimplex::new(12345);

        // Test octaves with different positions, frequencies and amplitudes
        let result1 = generate_octaves(&simplex, 0.0, 0.0, 4, 0.003, 1.0);
        let result2 = generate_octaves(&simplex, 10.0, 10.0, 4, 0.003, 1.0);
        let result3 = generate_octaves(&simplex, -10.0, -10.0, 4, 0.003, 1.0);

        // Result should be in [-1.0, 1.0]
        assert!(
            (-1.0..=1.0).contains(&result1),
            "Result should be in [-1.0, 1.0]"
        );
        assert!(
            (-1.0..=1.0).contains(&result2),
            "Result should be in [-1.0, 1.0]"
        );
        assert!(
            (-1.0..=1.0).contains(&result3),
            "Result should be in [-1.0, 1.0]"
        );

        // Different inputs should yield different results
        assert_ne!(
            result1, result2,
            "Different inputs should yield different results"
        );
    }

    #[test]
    fn test_generation_data_is_valid() {
        let data = create_test_data();

        // Check if the data is valid
        assert!(data.chunk_size > 0, "Chunk size should be positive");
        assert!(data.dirt_depth > 0, "Dirt depth should be positive");
    }

    #[test]
    fn test_chunk_has_correct_size() {
        let chunk = create_test_chunk();
        let data = create_test_data();
        let expected_blocks = (data.chunk_size * data.chunk_size) as usize;

        assert!(
            chunk.blocks.len() >= expected_blocks,
            "Chunk should habe at least {} blocks, has {}",
            expected_blocks,
            chunk.blocks.len()
        );
    }

    #[test]
    fn test_multiple_chunks_different_positions() {
        let data = create_test_data();
        let chunk1 = Chunk::new(&data, Vector2 { x: 0, y: 0 });
        let chunk2 = Chunk::new(&data, Vector2 { x: 1, y: 0 });

        // Different Chunks should have different Blocks
        assert_ne!(
            chunk1.blocks.len(),
            chunk2.blocks.len(),
            "Different Chunks should have different Blocks"
        );

        // Position of Chunks should be different
        assert_ne!(
            chunk1.position, chunk2.position,
            "Position of Chunks should be different"
        );
    }
}
