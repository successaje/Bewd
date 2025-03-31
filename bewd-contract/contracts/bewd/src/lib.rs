#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, token, vec, Env, String, Vec, Address, Map};

mod BewdToken {
    soroban_sdk::contractimport!(
            file = "/Users/finisher/Documents/github/stellar/Bewd/bewd-token/target/wasm32-unknown-unknown/release/bewd_token.wasm"
    );
}

const BewdWASM : &[u8] = include_bytes!("/Users/finisher/Documents/github/stellar/Bewd/bewd-token/target/wasm32-unknown-unknown/release/bewd_token.wasm");

#[contract]
pub struct BEWDContract;


#[contractimpl]
impl BEWDContract {

    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&"admin", &admin);
    }

    /// Create a user profile
    pub fn create_profile(env: Env, user: Address, username: String) {
        let mut profiles: Map<Address, String> = env.storage().instance().get(&"profiles").unwrap_or_default();
        profiles.set(user.clone(), username);
        env.storage().instance().set(&"profiles", &profiles);
    }

    /// Follow another user
    pub fn follow(env: Env, follower: Address, followee: Address) {
        let mut follows: Map<Address, Vec<Address>> = env.storage().instance().get(&"follows").unwrap_or_default();
        let user_follows = follows.get(follower.clone()).unwrap_or_default();
        follows.set(follower, user_follows.push_back(followee));
        env.storage().instance().set(&"follows", &follows);
    }

    /// Unfollow a user
    pub fn unfollow(env: Env, follower: Address, followee: Address) {
        let mut follows: Map<Address, Vec<Address>> = env.storage().instance().get(&"follows").unwrap_or_default();
        if let Some(mut user_follows) = follows.get(follower.clone()) {
            user_follows.retain(|x| x != &followee);
            follows.set(follower, user_follows);
        }
        env.storage().instance().set(&"follows", &follows);
    }

    /// Send a direct message
    pub fn send_message(env: Env, sender: Address, receiver: Address, message: String) {
        let mut messages: Map<(Address, Address), Vec<String>> = env.storage().instance().get(&"messages").unwrap_or_default();
        let user_messages = messages.get((sender.clone(), receiver.clone())).unwrap_or_default();
        messages.set((sender, receiver), user_messages.push_back(message));
        env.storage().instance().set(&"messages", &messages);
    }

    /// Create a social post
    pub fn create_post(env: Env, author: Address, content: String) {
        let mut posts: Vec<(Address, String)> = env.storage().instance().get(&"posts").unwrap_or_default();
        posts.push_back((author, content));
        env.storage().instance().set(&"posts", &posts);
    }
   
//    Tokenizes a post by creating shards (fractional tokens)
    pub fn tokenize_post(
        env: Env,
        creator: Address,
        post_id: String,
        total_shards: u32,
        metadata_uri: String,
    ) -> Address {
        // Verify creator signature
        creator.require_auth();
        
        // Create a new token contract for this post's shards
        let shard_token = token::StellarAssetClient::new(&env, &env.register_stellar_asset_contract(creator.clone()));
        shard_token.initialize(&creator, &7, &post_id, &String::from_slice(&env, "SHRD"));
        
        // Store post metadata and shard info
        let post_info = PostInfo {
            creator: creator.clone(),
            total_shards,
            claimed_shards: 0,
            metadata_uri,
            shard_token: shard_token.address.clone(),
        };
        
        env.storage().persistent().set(&post_id, &post_info);
        
        shard_token.address
    }

    // Allows users to "build" (invest in) a post by purchasing shards
    pub fn build_post(
        env: Env,
        builder: Address,
        post_id: String,
        shard_amount: u32,
        payment_amount: i128,
        bewd_token: Address,
    ) {
        // Verify builder signature
        builder.require_auth();
        
        // Get post info
        let post_info: PostInfo = env.storage().persistent().get(&post_id).unwrap();
        
        // Check available shards
        if post_info.claimed_shards + shard_amount > post_info.total_shards {
            panic!("Not enough shards available");
        }
        
        // Transfer BEWD tokens from builder to creator
        let token_client = token::Client::new(&env, &bewd_token);
        token_client.transfer(&builder, &post_info.creator, &payment_amount);
        
        // Mint post shard tokens to builder
        let shard_token = token::Client::new(&env, &post_info.shard_token);
        shard_token.mint(&builder, &(shard_amount as i128));
        
        // Update claimed shards count
        let mut updated_info = post_info;
        updated_info.claimed_shards += shard_amount;
        env.storage().persistent().set(&post_id, &updated_info);
    }

    // Gets post information
    pub fn get_post_info(env: Env, post_id: String) -> PostInfo {
        env.storage().persistent().get(&post_id).unwrap()
    }

     pub fn initialize(env: Env, admin: Address, token_name: String, token_symbol: String) -> Address {
        // Create the BEWD token
        let token = token::StellarAssetClient::new(&env, &env.create_asset_contract(admin.clone()));
        token.initialize(&admin, &7, &token_name, &token_symbol);
        
        admin
    }

    pub fn transfer(env: &Env, to: Address, amount: i128) {
        let token_client = token::Client::new(&env, &env.current_contract_address());
        token_client.transfer(&env.current_contract_address(), &to, &amount);
    }

    pub fn approve(env: &Env, owner: Address, spender: Address, amount: i128, expiration: u32) {
        let token_client = token::Client::new(&env, &env.current_contract_address());
        token_client.approve(&owner, &spender, &amount, &expiration);
    }
}

    
// }

// Data structure for post information
#[contracttype]
pub struct PostInfo {
    pub creator: Address,
    pub total_shards: u32,
    pub claimed_shards: u32,
    pub metadata_uri: String,
    pub shard_token: Address,
}


// 1️⃣ Integrate tipping logic (allow users to BEWD posts).
// 2️⃣ Implement fractionalized post ownership (allow users to buy post shares).
// 3️⃣ Set up revenue distribution (automatically reward shareowners when a post receives BEWDs).
mod test;
