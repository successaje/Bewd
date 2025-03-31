use crate::{
    admin::{read_administrator, write_administrator},
    allowance::{read_allowance, spend_allowance, write_allowance},
    approval::{read_approval, spend_approval, write_approval},
    balance::{read_balance, receive_balance, spend_balance},
    metadata::{read_decimal, read_name, read_symbol, write_metadata},
    fractional::{read_shard_count, write_shard_count, read_shard_owners, write_shard_owners, map_shards_to_post, get_post_for_shard},
    storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, DataKey},
};
use soroban_sdk::{
    contract, contractimpl, token, Address, Bytes, Env, String, Vec, Map, TryFromVal, Val, IntoVal,
};
use soroban_token_sdk::{TokenUtils, TokenMetadata};

#[derive(Clone, Debug, PartialEq)]
pub struct PostConfig {
    pub threshold: u32,
    pub build_count: u32,
    pub is_rwa: bool,
}

impl TryFromVal<Env, PostConfig> for PostConfig {
    type Error = soroban_sdk::Error;

    fn try_from_val(_env: &Env, v: &Val) -> Result<Self, Self::Error> {
        Ok(v.try_into()?)
    }
}

impl IntoVal<Env, Val> for PostConfig {
    fn into_val(self, _env: &Env) -> Val {
        (self.threshold, self.build_count, self.is_rwa).into_val(_env)
    }
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}



#[contract]
pub struct BewdNft;

#[contractimpl]
impl BewdNft {
    // Initializes a new fractionalizable post
    pub fn initialize_post(
        e: Env,
        admin: Address,
        post_id: Bytes,
        decimal: u32,
        name: String,
        symbol: String,
        threshold: u32,
        total_shards: u32,
        is_rwa: bool,
    ) {
        admin.require_auth();
        if decimal > 18 {
            panic!("Decimal must not be greater than 18");
        }

        write_administrator(&e, &admin);
        write_metadata(
            &e,
            TokenMetadata {
                decimal,
                name,
                symbol,
            },
        );

        // Initialize BEWD-specific config
        e.storage().instance().set(
            &DataKey::PostConfig(post_id.clone()),
            &PostConfig {
                threshold,
                build_count: 0,
                is_rwa,
            },
        );

        // Initialize shard supply
        write_shard_count(&e, post_id.clone(), total_shards);

        // Creator owns all shards initially
        let mut owners = Vec::new(&e);
        for _ in 0..total_shards {
            owners.push_back(admin.clone());
        }
        write_shard_owners(&e, post_id.clone(), owners);

        // Map all shards to this post
        map_shards_to_post(&e, post_id.clone(), total_shards);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        TokenUtils::new(&e).events().mint(admin, post_id, total_shards as i128);
    }

    // Records social engagement
    pub fn build_post(e: Env, post_id: Bytes, builder: Address) {
        builder.require_auth();

        let mut config: PostConfig = e
            .storage()
            .instance()
            .get(&DataKey::PostConfig(post_id.clone()))
            .unwrap();

        config.build_count += 1;
        e.storage()
            .instance()
            .set(&DataKey::PostConfig(post_id.clone()), &config);

        // Auto-fractionalize if threshold reached
        if config.build_count >= config.threshold {
            TokenUtils::new(&e)
                .events()
                .approve(post_id.clone(), builder, 1, 0); // Special approval event
        }

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }

    pub fn transfer_shard(
        e: Env,
        from: Address,
        to: Address,
        post_id: Bytes,
        shard_index: u32
    ) {
        from.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Update shard ownership
        let mut owners = read_shard_owners(&e, post_id.clone());
        assert_eq!(owners.get(shard_index).unwrap(), from, "Not shard owner");
        owners.set(shard_index, to.clone());
        write_shard_owners(&e, post_id, owners);
    }
}

#[contractimpl]
impl token::Interface for BewdNft {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_approval(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_approval(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    // Modified transfer for fractional shards
    fn transfer(e: Env, from: Address, to: Address, shard_index: i128) {
        from.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), 1);
        receive_balance(&e, to.clone(), 1);

        // Update shard ownership
        let post_id = get_post_for_shard(&e, shard_index);
        let mut owners = read_shard_owners(&e, post_id.clone());
        owners.set(shard_index as u32, to.clone());
        write_shard_owners(&e, post_id, owners);

        TokenUtils::new(&e).events().transfer(from, to, shard_index);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
    }


    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}
