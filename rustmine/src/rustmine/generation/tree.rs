use crate::rustmine::generation::types::BlockType;
use cgmath::Vector3;
use std::collections::HashMap;

pub(crate) fn generate_tree(
    blocks: &mut HashMap<Vector3<i32>, BlockType>,
    x: i32,
    base_y: i32,
    z: i32,
    seed: u32,
) {
    let roll = hash_pos(x, z, seed) % 100;
    if roll < 70 {
        generate_standard_oak(blocks, x, base_y, z);
    } else if roll < 85 {
        generate_grand_oak(blocks, x, base_y, z);
    } else if roll < 95 {
        generate_birch(blocks, x, base_y, z);
    } else {
        generate_spruce(blocks, x, base_y, z);
    }
}

/// Standard oak (original shape, unchanged).
fn generate_standard_oak(
    blocks: &mut HashMap<Vector3<i32>, BlockType>,
    x: i32,
    base_y: i32,
    z: i32,
) {
    for dy in 1..=4 {
        blocks.insert(
            Vector3 {
                x,
                y: base_y + dy,
                z,
            },
            BlockType::Oak,
        );
    }
    let layers: &[(i32, i32, bool)] = &[(3, 2, true), (4, 2, true), (5, 1, false), (6, 1, true)];
    insert_leaf_layers(blocks, x, base_y, z, layers, BlockType::LeavesOak);
}

/// Large oak: branching trunk with leaf clusters at each branch tip
fn generate_grand_oak(blocks: &mut HashMap<Vector3<i32>, BlockType>, x: i32, base_y: i32, z: i32) {
    for dy in 1..=5 {
        blocks.insert(
            Vector3 {
                x,
                y: base_y + dy,
                z,
            },
            BlockType::Oak,
        );
    }
    // Main canopy ball centered at trunk top
    insert_sphere(blocks, x, base_y + 6, z, 2, BlockType::LeavesOak);
    // Slightly offset secondary cluster for an organic silhouette
    insert_sphere(blocks, x + 1, base_y + 7, z, 2, BlockType::LeavesOak);
    insert_sphere(blocks, x - 1, base_y + 6, z + 1, 1, BlockType::LeavesOak);
}

/// Slender Birch: thin trunk (5 tall), two compact spherical canopy clusters.
fn generate_birch(blocks: &mut HashMap<Vector3<i32>, BlockType>, x: i32, base_y: i32, z: i32) {
    for dy in 1..=5 {
        blocks.insert(
            Vector3 {
                x,
                y: base_y + dy,
                z,
            },
            BlockType::Birch,
        );
    }
    // Main canopy ball centered at trunk top
    insert_sphere(blocks, x, base_y + 6, z, 2, BlockType::LeavesBirch);
    // Slightly offset secondary cluster for an organic silhouette
    insert_sphere(blocks, x + 1, base_y + 7, z, 2, BlockType::LeavesBirch);
    insert_sphere(blocks, x - 1, base_y + 6, z + 1, 1, BlockType::LeavesBirch);
}

/// Spruce: tall trunk (8), tight stacked layers narrowing to a point.
fn generate_spruce(blocks: &mut HashMap<Vector3<i32>, BlockType>, x: i32, base_y: i32, z: i32) {
    // Trunk
    for dy in 1..=8 {
        blocks.insert(
            Vector3 {
                x,
                y: base_y + dy,
                z,
            },
            BlockType::Spruce,
        );
    }

    // Layers: (dy, radius, clip_corners)
    // Wide base, then alternating radius-2/1 stepping up, tight cap
    let layers: &[(i32, i32, bool)] = &[
        (4, 2, false),
        (5, 2, true),
        (6, 2, false),
        (7, 1, false),
        (8, 1, false),
        (9, 1, true),
        (10, 0, false), // single tip
    ];
    insert_leaf_layers(blocks, x, base_y, z, layers, BlockType::LeavesSpruce);
}

// helpers
fn insert_leaf_layers(
    blocks: &mut HashMap<Vector3<i32>, BlockType>,
    x: i32,
    base_y: i32,
    z: i32,
    layers: &[(i32, i32, bool)],
    leaf: BlockType,
) {
    for &(dy, radius, clip_corners) in layers {
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if clip_corners && dx.abs() == radius && dz.abs() == radius {
                    continue;
                }
                blocks
                    .entry(Vector3 {
                        x: x + dx,
                        y: base_y + dy,
                        z: z + dz,
                    })
                    .or_insert(leaf);
            }
        }
    }
}

/// Fills a Manhattan-distance sphere of `radius` centered at (cx, cy, cz).
fn insert_sphere(
    blocks: &mut HashMap<Vector3<i32>, BlockType>,
    cx: i32,
    cy: i32,
    cz: i32,
    radius: i32,
    leaf: BlockType,
) {
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                if (dx * dx + dy * dy + dz * dz) <= radius * radius + radius {
                    blocks
                        .entry(Vector3 {
                            x: cx + dx,
                            y: cy + dy,
                            z: cz + dz,
                        })
                        .or_insert(leaf);
                }
            }
        }
    }
}

pub fn hash_pos(x: i32, z: i32, seed: u32) -> u32 {
    let mut h = 2166136261u32;
    for b in x
        .to_le_bytes()
        .iter()
        .chain(z.to_le_bytes().iter())
        .chain(seed.to_le_bytes().iter())
    {
        h ^= *b as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}
