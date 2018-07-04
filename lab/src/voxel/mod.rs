// Design goals:
// - Memory-efficient (assumes voxels are sparse (very much so));
// - Fast insertion;
// - Faster key retrieval (for traversal by rays);
// - Designed as a giant fixed-size buffer, to be directly transferred/mapped to GPU memory;
// - Code is easy to port to computes shaders;

use std::mem;
use std::slice;
use std::marker::PhantomData;
use vek::Vec3;

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
