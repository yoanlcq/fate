// # Problems
//
// ## Physics
//
// I want fast ray-casting in the whole scene's geometry.
// Usages include:
// - Terrain collision detection;
// - Collision _prediction_.
//
// ## Rendering
//
// I don't want to use cascaded shadow maps. Reasons are:
// - It's kinda slow and wasteful.
// - It's hard to make it look right (i.e not pixelated, and choose a correct bias, etc);
// - It looks like a hierarchical data structure, which I don't like.
//
// I would like not having to use light maps. Reasons are:
// - The PBR pipeline more or less does this by itself;
// - I don't want to deal with static/dynamic duality of lights.
// - I would like to avoid having to _bake_ stuff.
// - Light maps are complicated to implement.
//
// Goals:
// - Precise hard shadows;
// - Nice-looking soft shadows;
// - In real-time, with world streaming.
//
// Domain knowledge:
// - The world is large, doesn't move, and is super sparse, especially since we're dealing with surfaces, not solids (i.e the interior of a mesh is just air).
// - A few objects move. 
// - A few lights move (e.g the Sun, or torches (adding noise to the position every frame)).
//
// # Idea
// - Have a super-low LOD version of the scene's geometry (NB: in which we only need to store vertex positions and normals); it's the one that will be used for ray-casting.
// - All of the high-LOD scene's triangles are already in GPU memory (large buffer already used by DrawIndirect())
// - The same is true for the low-LOD scene;
// - In the usual high-LOD scene rendering step via DrawIndirect(), care to write these in renderbuffers:
//   - Depth; used for knowing the fragment's 3D world position;
//   - Normal; used for discarding a ray early if the fragment is already in shadow.
// - For each fragment, cast a ray from the fragment's 3D world position towards the light source, WITHIN some in-GPU representation of the super-low LOD scene.
//   The first test is "if dot(fragment_normal, dir_to_light) < 0 { return; }". Indeed, if a fragment is facing away from the light, we know it's already in shadow.
//   If the ray intersects any triangle before reaching the light, then the fragment is shadowed.
// - The question is, how to make the intersection test fast enough for rendering every frame?
//
// # One solution
//
// Throughout the whole thing, I want:
// - Memory and speed statistics;
// - Ability to tweak settings in real time and see the impact via stats.
//
// Split the scene into "objects". Factors to take into account are: sparsity, moving or not, and level of detail (as in, distance from the camera).
// An example would be:
// - The main character; Not sparse, moving, and close to camera.
// - The nearby large cave entrance: Sparse, not moving, somewhat close to camera.
// - The surrounding desert: Sparse, not moving, somewhat close to camera.
// - The canyon in the faraway distance: Sparse, not moving, very far from camera.
//
// Each object is assigned a low-res 3D texture, representing a voxel grid.
// The voxel grid's world-space scale is big enough to contain the object no matter its current animation pose.
// The voxel grid's resolution should be kinda low, but adjusted depending on the object's sparsity, wanted level of detail, and the look of the bounding box.
// - Low res => less memory + less cells to traverse, but cells are less likely to be empty.
// - High res => cells are likely to be empty, but this costs more memory, and traversal has to go though more cells.
//
// There's a very few number of such grids (as few as there are "objects"), so we can store all the grids in linear arrays.
//
// For a given object, each voxel of its grid stores 0 to N triangles. (TODO: how exactly?)
//
// Then, the GPU ray-casting algorithm looks like this:
// - if dot(fragment_normal, dir_to_light) < 0 { return; }
// - Find the potentially-intersecting grids (using AABB tests).
// - Note that AABBs may contain one another. So we have to find the intersection in _each_ grid, and return only the closest.
//   But if our ray casting is a boolean query (= "is there anything between my origin and the light?"), then we can return as soon as we find any intersection.
// - For each grid, traverse voxels using 3DDDA.
// - When traversing a voxel, rasterize by hand the triangles it contains into a 1x1 buffer. If we find something, there's an intersection. Otherwise, we have to keep going.
//
// Q: In the large desert scene, if a ray has no intersection, it has to traverse a lot of cells.
// A: Not if the "objects" are chosen such that as few bounding boxes as needs are selected. There's no bounding box for the air above the sand!

// Design goals:
// - Memory-efficient (assumes voxels are sparse (very much so));
// - Fast insertion;
// - Faster key retrieval (for traversal by rays);
// - Designed as a giant fixed-size buffer, to be directly transferred/mapped to GPU memory;
// - Code is easy to port to computes shaders;

use std::mem;
use std::marker::PhantomData;
use math::Vec3;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VoxelHashMap<T> {
    mem: Vec<u8>,
    chunk_capacity: usize,
    bucket_capacity: usize,
    capacity: usize,
    nb_buckets: usize,
    _phantom_data: PhantomData<T>,
}

pub struct VoxelHashMapSettings {
    /// Maximum number of key-value pairs.
    pub capacity: usize,
    /// Number of buckets. Recommended is 512 because of 9-bit XYZ hash.
    pub nb_buckets: usize,
    /// Maximum number of chunk indices per bucket.
    pub bucket_capacity: usize,
    /// Maximum number of elements per chunk.
    pub chunk_capacity: usize,
}

impl<T> VoxelHashMap<T> {
    pub fn new(settings: &VoxelHashMapSettings) -> Self {
        let &VoxelHashMapSettings {
            capacity,
            nb_buckets,
            bucket_capacity,
            chunk_capacity,
        } = settings;

        let bucket_storage = nb_buckets * (4 + 2 * bucket_capacity);
        let pool_storage = capacity * (4 + mem::size_of::<T>());
        Self {
            mem: {
                let cap = bucket_storage + pool_storage;
                let mut mem = Vec::<u8>::with_capacity(cap);
                unsafe {
                    mem.set_len(cap); // Undefined content, woohoo
                }
                mem
            },
            capacity,
            nb_buckets,
            chunk_capacity,
            bucket_capacity,
            _phantom_data: PhantomData,
        }
    }
    /*
    pub fn insert(&self, key: Vec3<u32>) {
        let h = hash_x3y3z3(key) % self.nb_buckets as u32; // PERF: If max_bucket <= 512, no need to modulate
        let (nb_entries, chunk_index_slots) = self.bucket(h);
        let nb_chunk_index_slots = (nb_entries + self.chunk_capacity - 1) / self.chunk_capacity;
        let mut entry_idx = 0;
        for chunk in chunk_index_slots[..nb_chunk_index_slots].iter().map(|idx| self.chunk_by_index(idx)) {
            for i in 0..self.chunk_capacity {
                if key == chunk[i] {
                    let old = self.value_chunk_by_index(idx)[i];
                    // Found! TODO Replace.
                    return;
                }
                entry_idx += 1;
                if entry_idx >= nb_entries {
                    // Not found!
                    //
                    // atomic_inc nb_entries
                    // if i == self.chunk_capacity - 1 {
                    //     // allocate new chunk
                    //     atomic_inc(pool.nb_chunks);
                    //     atomic_set(chunk_index_slots[chunk_index_slot_index + 1], pool.nb_chunks);
                    //     i += 1;
                    // }
                    // atomic_set(chunk[(i+1) % chunk_capacity], value);
                    return;
                }
            }
        }
    }
    fn bucket(&self, h: u32) -> (u32, &[u16]) {
        let stride = 4 + 2 * self.bucket_capacity;
        let offset = h as usize * stride;
        unsafe {
            let nb_entries = *(&self.mem[offset] as *const _ as *const u32);
            let slots = slice::from_raw_parts(&self.mem[offset+4] as *const _ as *const u16, self.bucket_capacity);
            (nb_entries, slots)
        }
    }
    */
}

pub fn hash_x3y3z3(v: Vec3<u32>) -> u32 {
    ((v.x & 0b111) << 6) | ((v.y & 0b111) << 3) | (v.z & 0b111)
}
