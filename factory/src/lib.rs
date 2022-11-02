use std::str::FromStr;

use near_sdk::borsh::{
    self,
    BorshDeserialize,
    BorshSerialize,
};
use near_sdk::collections::LookupSet;
use near_sdk::{
  self,
  assert_one_yocto,
  env,
  near_bindgen,
  AccountId,
  Promise,
  Gas,
};

use chershare_resource::ResourceInitParams;

// Constants

const fn tgas(n: u64) -> Gas {
    Gas(n * 10u64.pow(12))
}
const CREATE_RESOURCE_GAS: Gas = tgas(65 + 5);
const STORAGE_PRICE_PER_BYTE: u128 = 10_u128.pow(19); 

// Smart Contract

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ChershareResourceFactory {
    /// The `Resources`s this `Factory` has produced.
    pub resources: LookupSet<String>,
    /// owner 
    pub owner_id: AccountId,
}

// ----------------------- contract interface modules ----------------------- //
impl Default for ChershareResourceFactory {
    fn default() -> Self {
        env::panic_str("Not initialized yet.");
    }
}

#[near_bindgen]
impl ChershareResourceFactory {
    pub fn assert_only_owner(&self) {
        assert_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only contract owner can call this method"
        );
    }

    pub fn assert_no_resource_with_id(
        &self,
        resource_id: &String,
    ) {
        assert!(
            !self.check_contains_resource(resource_id),
            "Resource with that ID already exists"
        );
    }

    /// If a `Resource` with `resource_id` has been produced by this `Factory`, return `true`.
    pub fn check_contains_resource(
        &self,
        resource_id: &String,
    ) -> bool {
        self.resources.contains(resource_id)
    }

    /// Get the `owner_id` of this `Factory`.
    pub fn get_owner(&self) -> &AccountId {
        &self.owner_id
    }


    /// Set a new `owner_id` for `Factory`.
    #[payable]
    pub fn set_chershare_factory_owner(
        &mut self,
        account_id: AccountId,
    ) {
        self.assert_only_owner();
        let account_id = account_id;
        assert_ne!(account_id, env::predecessor_account_id());
        self.owner_id = account_id;
    }

    // /// Handle callback of resource creation.
    // #[private]
    // pub fn on_create(
    //     &mut self,
    //     resource_creator_id: AccountId,
    //     metadata: NFTContractMetadata,
    //     owner_id: AccountId,
    //     resource_account_id: AccountId,
    //     attached_deposit: U128,
    // ) {
    //     let attached_deposit: u128 = attached_deposit.into();
    //     if is_promise_success() {
    //         // pay out self and update contract state
    //         self.resources.insert(&metadata.name);
    //         env::log_str(
    //             &MbResourceDeployData {
    //                 contract_metadata: metadata,
    //                 owner_id: owner_id.to_string(),
    //                 resource_id: resource_account_id.to_string(),
    //             }
    //             .serialize_event(),
    //         );
    //         Promise::new(self.owner_id.to_string().parse().unwrap())
    //             .transfer(attached_deposit - self.storage_bytes);
    //         // #[cfg(feature = "panic-test")]
    //         // env::panic_str("event.near_json_event().as_str()");
    //     } else {
    //         // Refunding resource cost creation to the resource creator
    //         Promise::new(resource_creator_id).transfer(attached_deposit - self.storage_bytes);
    //         env::log_str("failed resource deployment");
    //     }
    // }

    #[init(ignore_state)]
    pub fn new() -> Self {
        assert!(!env::state_exists());
        Self {
            resources: LookupSet::new(b"t".to_vec()),
            owner_id: env::predecessor_account_id(),
        }
    }

    #[private]
    pub fn on_create(
        &mut self,
        store_creator_id: AccountId,
        metadata: NFTContractMetadata,
        owner_id: AccountId,
        store_account_id: AccountId,
        attached_deposit: U128,
    ) {
        let attached_deposit: u128 = attached_deposit.into();
        if is_promise_success() {
            // pay out self and update contract state
            self.stores.insert(&metadata.name);
            env::log_str(
                &MbStoreDeployData {
                    contract_metadata: metadata,
                    owner_id: owner_id.to_string(),
                    store_id: store_account_id.to_string(),
                }
                .serialize_event(),
            );
            Promise::new(self.owner_id.to_string().parse().unwrap())
                .transfer(attached_deposit - self.store_cost);
            // #[cfg(feature = "panic-test")]
            // env::panic_str("event.near_json_event().as_str()");
        } else {
            // Refunding store cost creation to the store creator
            Promise::new(store_creator_id).transfer(attached_deposit - self.store_cost);
            env::log_str("failed store deployment");
        }
    }

    /// `create_resource` checks that the attached deposit is sufficient before
    /// parsing the given resource_id, validating no such resource subaccount exists yet
    /// and generates a new resource from the resource metadata.
    #[payable]
    pub fn create_resource(
        &mut self,
        name: String,
        owner_id: AccountId,
        resource_init_params: ResourceInitParams 
    ) -> Promise {
        self.assert_no_resource_with_id(&name);
        // prepare arguments as json byte vector
        let init_args = serde_json::to_vec(&resource_init_params)
        .unwrap();
        // ResourceId is only the subaccount. resource_account_id is the full near qualified name.
        // Note, validity checked in `NFTContractMetadata::new;` above.

        let resource_account_id =
            AccountId::from_str(&*format!("{}.{}", resource_init_params.id, env::current_account_id()))
                .unwrap();
        Promise::new(resource_account_id.clone())
            .create_account()
            .transfer(env::attached_deposit()) 
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(include_bytes!("../../target/wasm32-unknown-unknown/release/chershare_resource.wasm").to_vec())
            .function_call("new".to_string(), init_args, 0, CREATE_RESOURCE_GAS)
            .then(factory_self::on_create(
                env::predecessor_account_id(),
                metadata,
                owner_id,
                resource_account_id,
                env::attached_deposit().into(),
                env::current_account_id(),
                NO_DEPOSIT,
                gas::ON_CREATE_CALLBACK,
            ))
    }
}

// ------------------------ impls on external types ------------------------- //
// TODO: Why the trait? -> to be able to impl it in this crate
pub trait New {
    fn new(arg: Self) -> Self;
}

impl New for NFTContractMetadata {
    fn new(args: NFTContractMetadata) -> Self {
        let resource_account = format!("{}.{}", args.name, env::current_account_id());
        assert!(
            env::is_valid_account_id(resource_account.as_bytes()),
            "Invalid character in resource id"
        );
        assert!(args.symbol.len() <= 6);

        Self {
            spec: args.spec,
            name: args.name,
            symbol: args.symbol,
            icon: args.icon,
            base_uri: args.base_uri,
            reference: args.reference,
            reference_hash: args.reference_hash,
        }
    }
}
