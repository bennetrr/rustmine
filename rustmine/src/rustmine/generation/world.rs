use crate::rustmine::generation::chunk::{Chunk, ChunkGenerator, ChunkMesh};

use crate::engine::rendering::camera::Camera;
use crate::engine::rendering::instance::ChunkBlockGroup;
use crate::rustmine::generation::tree::{generate_tree, hash_pos};
use crate::rustmine::generation::types::{
    BlockPos, BlockType, CameraData, ChunkPos, ChunkResult, PlayerPos,
};
use crate::rustmine::generation::types::{CHUNK_SIZE, ChunkStatus};
use crate::rustmine::saves::Save;
use cgmath::{Vector2, Vector3};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub(crate) struct GenerationData {
    pub(crate) seed: u32,
    pub(crate) chunk_size: u32,
    pub(crate) dirt_depth: u32,
    pub(crate) cobblestone_min_y: i32,
}

impl GenerationData {
    pub fn new(seed: u32, chunk_size: u32, dirt_depth: u32, cobblestone_min_y: i32) -> Self {
        Self {
            seed,
            chunk_size,
            dirt_depth,
            cobblestone_min_y,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WorldMetaData {
    pub(crate) last_played: SystemTime,
}

impl WorldMetaData {
    pub fn new(last_played: SystemTime) -> Self {
        Self { last_played }
    }
}

// Disk-only types, never stored in memory
#[derive(Serialize, Deserialize)]
struct SubChunkData {
    sub_chunk_y: i32,
    blocks: Vec<Option<BlockType>>,
}

#[derive(Serialize, Deserialize)]
struct ChunkData {
    pos: ChunkPos,
    sub_chunks: Vec<SubChunkData>,
}

pub(crate) struct World {
    pub(crate) chunks: HashMap<ChunkPos, Chunk>,
    pub(crate) generation_data: Arc<GenerationData>,
    pending_chunks: HashSet<ChunkPos>,
    terrain_ready: HashSet<ChunkPos>,
    pending_tree_blocks: HashMap<ChunkPos, HashMap<BlockPos, BlockType>>,
    chunk_generator: ChunkGenerator,
}

impl World {
    /// Constructs a new World.
    pub fn new(world_name: &str) -> Self {
        let generation_data = Arc::new(
            World::load_world_settings(world_name).expect("failed to load generation settings"),
        );
        let chunks = Self::generate_chunks(&generation_data);
        let chunk_generator = ChunkGenerator::new(8, Arc::clone(&generation_data));
        Self {
            chunks,
            generation_data,
            pending_chunks: HashSet::new(),
            terrain_ready: HashSet::new(),
            pending_tree_blocks: HashMap::new(),
            chunk_generator,
        }
    }

    /// Generates a collection of chunks arranged in a square grid centered around the origin.
    ///
    /// # Arguments
    /// * `data` - Generation parameters, including the grid `size` (number of chunks per side).
    /// * `seed` - The seed used for procedural generation of each chunk.
    ///
    /// # Returns
    /// A `HashMap` mapping each [`ChunkPos`] to its generated [`Chunk`],
    /// covering positions in the range `(-size/2)...(size/2)` on both the X and Z axes.
    fn generate_chunks(data: &GenerationData) -> HashMap<ChunkPos, Chunk> {
        let mut chunks: HashMap<ChunkPos, Chunk> = HashMap::new();
        let half = 16 / 2; // chunk_pos: (-8..7), if without half - chunk_pos: (0..15)

        for pos_x in -half..half {
            for pos_z in -half..half {
                let chunk = Chunk::new(data, ChunkPos::new(pos_x, pos_z));
                chunks.insert(chunk.position, chunk);

                log::info!("[Generation] Generated Chunk at x: {}, z: {}", pos_x, pos_z);
            }
        }

        for pos_x in -half..half {
            for pos_z in -half..half {
                let columns: Vec<(i32, i32, i32)> = chunks[&ChunkPos::new(pos_x, pos_z)]
                    .blocks
                    .iter()
                    .filter(|(_, bt)| **bt == BlockType::Grass)
                    .map(|(p, _)| (p.x, p.y, p.z))
                    .collect();

                for (x, y, z) in columns {
                    if hash_pos(x, z, data.seed).is_multiple_of(137) {
                        let mut tree_blocks = HashMap::new();
                        generate_tree(&mut tree_blocks, x, y, z, data.seed);

                        if tree_blocks
                            .keys()
                            .map(|pos| Self::chunk_pos_from_block_pos(*pos))
                            .unique()
                            .map(|pos| chunks.get(&pos))
                            .any(|chunk| chunk.is_none())
                        {
                            log::warn!(
                                "Failed to generate tree on ({x}, {y}, {z}): Tree would be placed in ungenerated chunk"
                            );
                            continue;
                        }

                        for (block_pos, block_type) in tree_blocks {
                            let target_chunk = Self::chunk_pos_from_block_pos(block_pos);
                            if let Some(chunk) = chunks.get_mut(&target_chunk) {
                                chunk.set_block(block_pos, block_type);
                            }
                        }
                    }
                }
            }
        }

        chunks
    }

    /// Requests generation of all chunks within render distance that are not yet loaded,
    /// pending, or awaiting decoration.
    ///
    /// Iterates over a square grid of chunk positions centered on the player and submits
    /// a generation request via [`ChunkGenerator::request`] for any position not already
    /// covered by one of:
    /// - `pending_chunks` — already queued with the worker pool,
    /// - `chunk_groups` — already has GPU data loaded,
    /// - `terrain_ready` — terrain has finished and the chunk is awaiting decoration.
    ///
    /// Newly requested positions are tracked in `pending_chunks` to avoid duplicate requests
    /// on subsequent frames.
    ///
    /// Call this once per frame before [`collect_generated_chunks`].
    ///
    /// # Arguments
    ///
    /// * `player_position` - Current player position in block-space; converted internally to
    ///   chunk-space to find the grid center.
    /// * `render_distance_radius` - Half-extent of the square, in chunks. A value of `n`
    ///   produces a `(2n + 1) × (2n + 1)` grid.
    /// * `chunk_groups` - Currently loaded GPU chunk groups; positions present here are
    ///   already fully loaded and do not need to be re-requested.
    pub fn request_chunks_in_runtime(
        &mut self,
        player_position: PlayerPos,
        render_distance_radius: i32,
        chunk_groups: &HashMap<ChunkPos, HashMap<BlockType, ChunkBlockGroup>>,
    ) {
        let chunk_pos = Self::chunk_pos_from_block_pos(BlockPos::new(
            player_position.x as i32,
            player_position.y as i32,
            player_position.z as i32,
        ));

        for pos_x in (chunk_pos.x - render_distance_radius)..=(chunk_pos.x + render_distance_radius)
        {
            for pos_z in
                (chunk_pos.y - render_distance_radius)..=(chunk_pos.y + render_distance_radius)
            {
                let pos = ChunkPos::new(pos_x, pos_z);

                if !self.pending_chunks.contains(&pos)
                    && !chunk_groups.contains_key(&pos)
                    && !self.terrain_ready.contains(&pos)
                {
                    self.chunk_generator.request(pos);
                    self.pending_chunks.insert(pos);
                }
            }
        }
    }

    /// Collects finished chunk-generation work and advances chunks through the
    /// terrain -> decoration pipeline.
    ///
    /// Drains completed work from the worker pool via [`ChunkGenerator::poll_ready`].
    /// For each finished chunk, the position is removed from `pending_chunks` and the
    /// [`Chunk`] is inserted into `self.chunks`:
    /// - [`ChunkStatus::Generated`] (terrain only) chunks are marked ready for decoration
    ///   in `terrain_ready`.
    /// - [`ChunkStatus::Decorated`] chunks are considered complete, and their
    ///   [`ChunkMesh`] is added to the returned list.
    ///
    /// After processing completions, any chunk in `terrain_ready` whose neighbours are
    /// all terrain-ready as well (see [`all_neighbours_ready`]) is promoted to the
    /// decoration phase: trees are procedurally placed on its grass columns (deterministic
    /// per-column via `hash_pos`), and the updated chunk is sent back to the worker pool
    /// for remeshing. These chunks are re-inserted into `pending_chunks` and their
    /// resulting meshes will surface on a future call once decoration/remeshing completes
    /// — not on this call.
    ///
    /// Call this once per frame after [`request_chunks_in_runtime`]. The returned meshes
    /// must be uploaded to the GPU by the caller before the next renderpass.
    ///
    /// # Returns
    ///
    /// A [`Vec<ChunkMesh>`] containing one mesh per chunk that finished the *decoration*
    /// phase this frame, in completion order. Chunks that only finished terrain
    /// generation this frame are not included (they're queued for decoration instead).
    /// Returns an empty [`Vec`] if no chunks completed decoration this frame.
    pub fn collect_generated_chunks(&mut self) -> Vec<ChunkMesh> {
        let mut meshes = Vec::new();

        for result in self.chunk_generator.poll_ready() {
            match result {
                ChunkResult::Generated(chunk, tree_blocks) => {
                    self.pending_chunks.remove(&chunk.position);
                    self.terrain_ready.insert(chunk.position);
                    self.pending_tree_blocks.insert(chunk.position, tree_blocks);
                    self.chunks.insert(chunk.position, chunk);
                }
                ChunkResult::Decorated(pos, mesh) => {
                    self.pending_chunks.remove(&pos);
                    // Do NOT overwrite the chunk's blocks with the remesh result: the
                    // worker meshed a snapshot taken when decoration was queued, and a
                    // neighbour may have written cross-border tree blocks into this
                    // chunk since then. Keep `self.chunks` authoritative, just flip the
                    // status and mark it dirty so `rebuild_block_groups` re-meshes from
                    // the real block data if anything changed after the snapshot.
                    if let Some(existing) = self.chunks.get_mut(&pos) {
                        existing.status = ChunkStatus::Decorated;
                        existing.mark_as_dirty();
                    }
                    meshes.push(mesh);
                }
            }
        }

        let candidates: Vec<ChunkPos> = self
            .terrain_ready
            .iter()
            .copied()
            .filter(|pos| self.all_neighbours_ready(*pos))
            .collect();

        for pos in candidates {
            self.terrain_ready.remove(&pos);

            let tree_blocks = self.pending_tree_blocks.remove(&pos).unwrap_or_default();

            let all_targets_ready = tree_blocks
                .keys()
                .map(|p| Self::chunk_pos_from_block_pos(*p))
                .all(|target| self.chunks.contains_key(&target));

            if all_targets_ready {
                for (block_pos, block_type) in tree_blocks {
                    self.set_block(block_pos, block_type);
                }
            } else {
                log::warn!(
                    "Skipped tree placement for chunk {pos:?}: a target chunk isn't generated yet"
                );
            }

            let blocks = self.chunks[&pos].blocks.clone();
            self.chunk_generator.remesh_tx.send((pos, blocks)).ok();
            self.pending_chunks.insert(pos);
        }

        meshes
    }

    fn all_neighbours_ready(&self, pos: ChunkPos) -> bool {
        for dx in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let neighbour = ChunkPos::new(pos.x + dx, pos.y + dz);
                if !self.chunks.contains_key(&neighbour) && !self.terrain_ready.contains(&neighbour)
                {
                    return false;
                }
            }
        }
        true
    }

    /// Converts a block position to the chunk position that contains it.
    fn chunk_pos_from_block_pos(pos: BlockPos) -> ChunkPos {
        Vector2::new(
            pos.x.div_euclid(16), // CHUNK_SIZE
            pos.z.div_euclid(16), // CHUNK_SIZE
        )
    }

    pub(crate) fn chunk_pos_for_block_pos(&self, pos: BlockPos) -> ChunkPos {
        Self::chunk_pos_from_block_pos(pos)
    }

    /// Returns the chunk position for `pos` if that chunk is loaded, otherwise `None`.
    pub(crate) fn find_chunk_pos_for_block_pos(&self, pos: BlockPos) -> Option<ChunkPos> {
        let chunk_pos = Self::chunk_pos_from_block_pos(pos);

        self.chunks.contains_key(&chunk_pos).then_some(chunk_pos)
    }

    pub(crate) fn get_block(&self, pos: BlockPos) -> Option<&BlockType> {
        let chunk_pos = Self::chunk_pos_from_block_pos(pos);

        self.chunks.get(&chunk_pos)?.get_block(pos)
    }

    pub(crate) fn get_y(&self, pos: Vector2<i32>) -> i32 {
        let chunk_pos = Self::chunk_pos_from_block_pos(BlockPos::new(pos.x, 0, pos.y));

        self.chunks.get(&chunk_pos).unwrap().get_y(pos)
    }

    pub(crate) fn set_block(&mut self, pos: BlockPos, block_type: BlockType) -> Option<ChunkPos> {
        self.mark_neighboring_chunks_as_dirty(pos);

        let chunk_pos = Self::chunk_pos_from_block_pos(pos);
        let chunk = self.chunks.get_mut(&chunk_pos)?;

        chunk.set_block(pos, block_type);

        Some(chunk_pos)
    }

    pub(crate) fn remove_block(&mut self, pos: BlockPos) -> Option<(ChunkPos, BlockType)> {
        self.mark_neighboring_chunks_as_dirty(pos);

        let chunk_pos = Self::chunk_pos_from_block_pos(pos);
        let chunk = self.chunks.get_mut(&chunk_pos)?;

        chunk
            .remove_block(pos)
            .map(|block_type| (chunk_pos, block_type))
    }

    fn mark_neighboring_chunks_as_dirty(&mut self, pos: BlockPos) {
        let chunk_pos = Self::chunk_pos_from_block_pos(pos);
        let x_pos_in_chunk = pos.x.rem_euclid(16);
        let z_pos_in_chunk = pos.z.rem_euclid(16);

        if x_pos_in_chunk == 0 {
            let neighbor_pos = ChunkPos::new(chunk_pos.x - 1, chunk_pos.y);
            if let Some(neighbor_chunk) = self.chunks.get_mut(&neighbor_pos) {
                neighbor_chunk.mark_as_dirty();
            }
        } else if x_pos_in_chunk == 15 {
            let neighbor_pos = ChunkPos::new(chunk_pos.x + 1, chunk_pos.y);
            if let Some(neighbor_chunk) = self.chunks.get_mut(&neighbor_pos) {
                neighbor_chunk.mark_as_dirty();
            }
        }

        if z_pos_in_chunk == 0 {
            let neighbor_pos = ChunkPos::new(chunk_pos.x, chunk_pos.y - 1);
            if let Some(neighbor_chunk) = self.chunks.get_mut(&neighbor_pos) {
                neighbor_chunk.mark_as_dirty();
            }
        } else if z_pos_in_chunk == 15 {
            let neighbor_pos = ChunkPos::new(chunk_pos.x, chunk_pos.y + 1);
            if let Some(neighbor_chunk) = self.chunks.get_mut(&neighbor_pos) {
                neighbor_chunk.mark_as_dirty();
            }
        }
    }

    /// Serializes the chunk map and writes it to the world's chunks-file.
    ///
    /// # Errors
    /// Returns an [`std::io::Error`] if the file cannot be written.
    pub(crate) fn save_world(
        &self,
        world: &HashMap<ChunkPos, Chunk>,
        world_name: &str,
    ) -> std::io::Result<()> {
        let path = Save::get_by_name(world_name).get_chunks_file();
        log::info!("Saving world to {}", path.display());

        let disk_world: Vec<ChunkData> = world.values().map(chunk_to_disk).collect();
        let bytes = postcard::to_allocvec(&disk_world).expect("serialize failed");
        let compressed = zstd::encode_all(&bytes[..], 3).expect("compression failed");

        fs::write(path, compressed)
    }

    /// Reads and deserializes the world's chunks file into a chunk map.
    ///
    /// Returns `None` if the file cannot be read or deserialization fails.
    pub(crate) fn load_world(world_name: &str) -> Option<HashMap<ChunkPos, Chunk>> {
        let path = Save::get_by_name(world_name).get_chunks_file();
        log::info!("Loading world from {}", path.display());
        let compressed = fs::read(path).ok()?;
        let bytes = zstd::decode_all(&compressed[..]).ok()?;

        let disk_world: Vec<ChunkData> = postcard::from_bytes(&bytes).ok()?;
        Some(
            disk_world
                .into_iter()
                .map(|cd| (cd.pos, disk_to_chunk(cd)))
                .collect(),
        )
    }

    pub(crate) fn save_world_metadata(
        world_name: &str,
        last_played: SystemTime,
    ) -> std::io::Result<()> {
        let meta = WorldMetaData::new(last_played);
        let meta_bytes = postcard::to_allocvec(&meta).expect("serialize failed");
        fs::write(
            Save::get_by_name(world_name).get_world_metadata_file(),
            meta_bytes,
        )
    }
    pub(crate) fn load_world_metadata(world_name: &str) -> Option<WorldMetaData> {
        let path = Save::get_by_name(world_name).get_world_metadata_file();
        log::info!("Loading world metadata from {}", path.display());

        let bytes = fs::read(path).ok()?;
        postcard::from_bytes(&bytes).ok()
    }

    /// Serializes the world settings and writes it to the world_gen_settings-file.
    ///
    /// # Errors
    /// Returns an [`std::io::Error`] if the file cannot be written.
    pub(crate) fn save_world_settings(
        world_name: &str,
        data: &GenerationData,
    ) -> std::io::Result<()> {
        let path = Save::get_by_name(world_name).get_world_gen_settings_file();
        log::info!("Saving world settings to {}", path.display());
        let bytes = postcard::to_allocvec(data).expect("serialize failed");
        fs::write(path, bytes)
    }

    /// Reads and deserializes the world_gen_settings-file into a GenerationData struct.
    ///
    /// Returns `None` if the file cannot be read or deserialization fails.
    pub(crate) fn load_world_settings(world_name: &str) -> Option<GenerationData> {
        let path = Save::get_by_name(world_name).get_world_gen_settings_file();
        log::info!("Loading world settings from {}", path.display());
        let bytes = fs::read(path).ok()?;
        postcard::from_bytes(&bytes).ok()
    }

    /// Serializes the camera state `(eye, target, up)` and writes it to the world's camera file.
    ///
    /// Creates the save directory if it does not exist.
    ///
    /// # Errors
    /// Returns an [`std::io::Error`] if directory creation or the file write fails.
    pub(crate) fn save_camera(
        &self,
        camera: &Camera,
        world_name: &str,
        is_creative: bool,
    ) -> std::io::Result<()> {
        let data: CameraData = (
            camera.eye.into(),
            camera.target.into(),
            camera.up.into(),
            is_creative,
        );
        let path = Save::get_by_name(world_name).create()?.get_camera_file();
        let bytes = postcard::to_allocvec(&data).expect("serialize failed");
        fs::write(path, bytes)
    }

    /// Reads and deserializes the world's camera file.
    ///
    /// Returns `None` if the file cannot be read or deserialization fails.
    ///
    /// # Returns
    /// `(eye, target, up)` as `[f32; 3]` arrays.
    pub(crate) fn load_camera(&self, world_name: &str) -> Option<CameraData> {
        let path = Save::get_by_name(world_name).get_camera_file();
        let bytes = fs::read(path).ok()?;
        postcard::from_bytes(&bytes).ok()
    }

    /// Creates a `World` from a pre-built chunk map without generating new chunks.
    pub fn from_chunks(chunks: HashMap<ChunkPos, Chunk>, world_name: &str) -> Self {
        let generation_data = Arc::new(World::load_world_settings(world_name).unwrap());
        let chunk_generator = ChunkGenerator::new(4, Arc::clone(&generation_data));

        Self {
            chunks,
            generation_data,
            pending_chunks: HashSet::new(),
            terrain_ready: HashSet::new(),
            pending_tree_blocks: HashMap::new(),
            chunk_generator,
        }
    }
}

// Save World Helpers
fn chunk_to_disk(chunk: &Chunk) -> ChunkData {
    let mut sub_map: HashMap<i32, [Option<BlockType>; 4096]> = HashMap::new();

    for (pos, block_type) in &chunk.blocks {
        let sub_y = pos.y.div_euclid(CHUNK_SIZE);
        let local_x = pos.x.rem_euclid(CHUNK_SIZE) as usize;
        let local_z = pos.z.rem_euclid(CHUNK_SIZE) as usize;
        let local_y = pos.y.rem_euclid(CHUNK_SIZE) as usize;
        let index = local_x + local_z * 16 + local_y * 256;

        sub_map.entry(sub_y).or_insert([None; 4096])[index] = Some(*block_type);
    }

    let sub_chunks = sub_map
        .into_iter()
        .filter(|(_, blocks)| blocks.iter().any(|b| b.is_some())) // cull all-air
        .map(|(sub_chunk_y, blocks)| SubChunkData {
            sub_chunk_y,
            blocks: blocks.to_vec(),
        })
        .collect();

    ChunkData {
        pos: chunk.position,
        sub_chunks,
    }
}

fn disk_to_chunk(data: ChunkData) -> Chunk {
    let mut blocks = HashMap::new();
    let chunk_base_x = data.pos.x * 16;
    let chunk_base_z = data.pos.y * 16;

    for sub in data.sub_chunks {
        let base_y = sub.sub_chunk_y * 16;

        for (index, slot) in sub.blocks.into_iter().enumerate() {
            let Some(block_type) = slot else { continue };

            let local_x = index % 16;
            let local_z = (index / 16) % 16;
            let local_y = index / 256;

            blocks.insert(
                Vector3::new(
                    chunk_base_x + local_x as i32,
                    base_y + local_y as i32,
                    chunk_base_z + local_z as i32,
                ),
                block_type,
            );
        }
    }

    Chunk {
        blocks,
        position: data.pos,
        status: ChunkStatus::Decorated,
        is_dirty: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test generation parameters, mirroring the defaults used at world creation.
    fn test_generation_data() -> GenerationData {
        GenerationData::new(12345, 16, 3, -64)
    }

    /// Builds a `World` directly from a chunk map without touching the disk.
    ///
    /// Mirrors [`World::from_chunks`], but uses in-memory generation settings so tests
    /// don't depend on saved world files.
    fn build_test_world(chunks: HashMap<ChunkPos, Chunk>) -> World {
        let generation_data = Arc::new(test_generation_data());
        let chunk_generator = ChunkGenerator::new(4, Arc::clone(&generation_data));

        World {
            chunks,
            generation_data,
            pending_chunks: HashSet::new(),
            terrain_ready: HashSet::new(),
            pending_tree_blocks: HashMap::new(),
            chunk_generator,
        }
    }

    /// Creates a controlled world with manually placed blocks (no noise generation)
    fn create_controlled_world() -> World {
        let mut chunks = HashMap::new();
        let mut chunk = Chunk::new(&test_generation_data(), Vector2::new(0, 0));

        // Manually add test blocks
        chunk.set_block(Vector3::new(0, 64, 0), BlockType::Grass);
        chunk.set_block(Vector3::new(0, 63, 0), BlockType::Dirt);
        chunk.set_block(Vector3::new(0, 62, 0), BlockType::Cobblestone);
        chunk.set_block(Vector3::new(10, 70, 10), BlockType::Oak);

        chunks.insert(Vector2::new(0, 0), chunk);
        build_test_world(chunks)
    }

    /// Creates an empty world with no chunks
    fn create_empty_world() -> World {
        build_test_world(HashMap::new())
    }

    #[test]
    fn test_empty_world_has_no_chunks() {
        let world = create_empty_world();
        assert!(world.chunks.is_empty(), "Empty world should have no chunks");
    }

    // BLOCK OPERATION TESTS
    #[test]
    fn test_get_block_on_existing_block() {
        let world = create_controlled_world();
        let pos = Vector3::new(0, 64, 0);

        let block = world.get_block(pos);
        assert!(block.is_some(), "Block at position {:?} should exist", pos);
        assert_eq!(*block.unwrap(), BlockType::Grass, "Block should be Grass");
    }

    #[test]
    fn test_get_block_on_non_existing_block() {
        let world = create_controlled_world();
        let pos = Vector3::new(9999, 9999, 9999);

        let block = world.get_block(pos);
        assert!(
            block.is_none(),
            "Block at position {:?} should NOT exist",
            pos
        );
    }

    #[test]
    fn test_set_block_overwrites_existing_block() {
        let mut world = create_controlled_world();
        let pos = Vector3::new(0, 64, 0);

        assert_eq!(*world.get_block(pos).unwrap(), BlockType::Grass);

        world.set_block(pos, BlockType::Oak);
        assert_eq!(
            *world.get_block(pos).unwrap(),
            BlockType::Oak,
            "Block should be overwritten with Oak"
        );
    }

    #[test]
    fn test_remove_block_removes_existing_block() {
        let mut world = create_controlled_world();
        let pos = Vector3::new(0, 64, 0);

        assert!(world.get_block(pos).is_some());

        let removed = world.remove_block(pos);
        assert!(removed.is_some(), "remove_block should return Some");
        assert_eq!(removed.unwrap().1, BlockType::Grass);

        assert!(world.get_block(pos).is_none(), "Block should be removed");
    }

    #[test]
    fn test_remove_block_on_non_existent_returns_none() {
        let mut world = create_controlled_world();
        let pos = Vector3::new(9999, 9999, 9999);

        let removed = world.remove_block(pos);
        assert!(
            removed.is_none(),
            "remove_block on non-existent block should return None"
        );
    }

    // CHUNK POSITION TESTS
    #[test]
    fn test_find_chunk_pos_for_block_pos() {
        let world = create_controlled_world();

        // Block at (5,64,5) is in chunk (0,0)
        let pos = Vector3::new(5, 64, 5);
        let chunk_pos = world.find_chunk_pos_for_block_pos(pos);
        assert_eq!(chunk_pos, Some(Vector2::new(0, 0)));
    }

    #[test]
    fn test_find_chunk_pos_for_out_of_bounds() {
        let world = create_controlled_world();

        // Block far outside the world
        let pos = Vector3::new(10000, 64, 10000);
        let chunk_pos = world.find_chunk_pos_for_block_pos(pos);
        assert!(
            chunk_pos.is_none(),
            "Out-of-bounds block should return None"
        );
    }

    #[test]
    fn test_chunk_pos_conversion_positive_coordinates() {
        let pos = Vector3::new(50, 64, 30);
        let chunk_pos = World::chunk_pos_from_block_pos(pos);

        assert_eq!(chunk_pos.x, 3, "Block X=50 should be in chunk X=3");
        assert_eq!(chunk_pos.y, 1, "Block Z=30 should be in chunk Z=1");
    }

    #[test]
    fn test_chunk_pos_conversion_negative_coordinates() {
        let pos = Vector3::new(-1, 64, -1);
        let chunk_pos = World::chunk_pos_from_block_pos(pos);

        assert_eq!(chunk_pos.x, -1, "Block X=-1 should be in chunk X=-1");
        assert_eq!(chunk_pos.y, -1, "Block Z=-1 should be in chunk Z=-1");
    }

    #[test]
    fn test_chunk_pos_conversion_at_boundary() {
        let pos = Vector3::new(16, 64, 32);
        let chunk_pos = World::chunk_pos_from_block_pos(pos);

        assert_eq!(chunk_pos.x, 1, "Block X=16 should be in chunk X=1");
        assert_eq!(chunk_pos.y, 2, "Block Z=32 should be in chunk Z=2");
    }

    // SAVE & LOAD TESTS (with postcard)
    #[test]
    fn test_load_non_existent_world_returns_none() {
        let world_name = "non_existent_world_12345".to_string();
        let loaded_chunks = World::load_world(&world_name);

        assert!(
            loaded_chunks.is_none(),
            "Loading non-existent world should return None"
        );
    }

    // INTEGRATION TESTS

    #[test]
    fn test_blocks_in_different_chunks() {
        let mut world = create_empty_world();

        // Add multiple chunks
        let chunk1 = Chunk::new(&test_generation_data(), Vector2::new(0, 0));
        let chunk2 = Chunk::new(&test_generation_data(), Vector2::new(1, 0));
        let chunk3 = Chunk::new(&test_generation_data(), Vector2::new(0, 1));

        world.chunks.insert(Vector2::new(0, 0), chunk1);
        world.chunks.insert(Vector2::new(1, 0), chunk2);
        world.chunks.insert(Vector2::new(0, 1), chunk3);

        // Blocks in different chunks
        let pos1 = Vector3::new(5, 64, 5); // Chunk (0,0)
        let pos2 = Vector3::new(20, 64, 5); // Chunk (1,0)
        let pos3 = Vector3::new(5, 64, 20); // Chunk (0,1)

        world.set_block(pos1, BlockType::Grass);
        world.set_block(pos2, BlockType::Dirt);
        world.set_block(pos3, BlockType::Oak);

        assert_eq!(*world.get_block(pos1).unwrap(), BlockType::Grass);
        assert_eq!(*world.get_block(pos2).unwrap(), BlockType::Dirt);
        assert_eq!(*world.get_block(pos3).unwrap(), BlockType::Oak);

        // Verify chunks are different
        let chunk_pos1 = world.find_chunk_pos_for_block_pos(pos1).unwrap();
        let chunk_pos2 = world.find_chunk_pos_for_block_pos(pos2).unwrap();
        let chunk_pos3 = world.find_chunk_pos_for_block_pos(pos3).unwrap();

        assert_ne!(chunk_pos1, chunk_pos2);
        assert_ne!(chunk_pos1, chunk_pos3);
    }

    // PERFORMANCE TESTS
    #[test]
    fn test_world_block_operations_performance() {
        let start = std::time::Instant::now();
        let mut world = create_empty_world();

        // Add a chunk
        let chunk = Chunk::new(&test_generation_data(), Vector2::new(0, 0));
        world.chunks.insert(Vector2::new(0, 0), chunk);

        // Perform many block operations
        for i in 0..500 {
            let pos = Vector3::new(i, 100, i);
            world.set_block(pos, BlockType::Grass);
        }

        for i in 0..500 {
            let pos = Vector3::new(i, 100, i);
            world.remove_block(pos);
        }

        let duration = start.elapsed();
        println!("500 block operations completed in {:?}", duration);

        // Performance assertion: should complete within reasonable time
        assert!(
            duration.as_millis() < 1000,
            "Block operations took too long: {:?}",
            duration
        );
    }
}
