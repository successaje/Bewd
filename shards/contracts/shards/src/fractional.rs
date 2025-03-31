use soroban_sdk::{Env, Bytes, Vec, Address};
use crate::storage_types::DataKey;

pub fn read_shard_count(e: &Env, post_id: Bytes) -> u32 {
    e.storage()
        .instance()
        .get(&DataKey::ShardCount(post_id))
        .unwrap_or(0)
}

pub fn write_shard_count(e: &Env, post_id: Bytes, count: u32) {
    e.storage().instance().set(&DataKey::ShardCount(post_id), &count);
}

pub fn read_shard_owners(e: &Env, post_id: Bytes) -> Vec<Address> {
    e.storage()
        .instance()
        .get(&DataKey::ShardOwners(post_id))
        .unwrap()
}

pub fn write_shard_owners(e: &Env, post_id: Bytes, owners: Vec<Address>) {
    e.storage()
        .instance()
        .set(&DataKey::ShardOwners(post_id), &owners);
}

pub fn get_post_for_shard(e: &Env, shard_index: u32) -> Bytes {
    // We need to maintain a mapping of shard indices to post IDs
    let shard_map: Map<u32, Bytes> = e.storage()
        .instance()
        .get(&DataKey::ShardToPostMap)
        .unwrap_or_else(|| Map::new(e));

    shard_map.get(shard_index)
        .unwrap_or_else(|| panic!("Shard {} not mapped to any post", shard_index))
}

// Also add this function to update the mapping when creating posts
pub fn map_shards_to_post(e: &Env, post_id: Bytes, total_shards: u32) {
    let mut shard_map: Map<u32, Bytes> = e.storage()
        .instance()
        .get(&DataKey::ShardToPostMap)
        .unwrap_or_else(|| Map::new(e));

    for shard_index in 0..total_shards {
        shard_map.set(shard_index, post_id.clone());
    }

    e.storage()
        .instance()
        .set(&DataKey::ShardToPostMap, &shard_map);
}