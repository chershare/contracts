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
  Gas,
  Promise, 
  PromiseError, 
};

use chershare_resource::ResourceInitParams;

// Constants

const fn tgas(n: u64) -> Gas {
  Gas(n * 10u64.pow(12))
}
const CREATE_RESOURCE_GAS: Gas = tgas(65 + 5);
// const STORAGE_PRICE_PER_BYTE: u128 = 10_u128.pow(19); 

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ChershareResourceFactory {
  /// The `Resources`s this `Factory` has produced.
  pub resources: LookupSet<String>,
  /// owner 
  pub owner_id: AccountId,
}

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


  #[init(ignore_state)]
  pub fn new() -> Self {
    assert!(!env::state_exists());
    Self {
      resources: LookupSet::new(b"t".to_vec()),
      owner_id: env::predecessor_account_id(),
    }
  }

  #[payable]
  pub fn create_resource(
    &mut self,
    name: String,
    resource_init_params: ResourceInitParams 
  ) -> Promise {
    self.assert_no_resource_with_id(&name);
    // prepare arguments as json byte vector
    let init_args = format!(
      "{{ \"resouce_init_params\": {} }}", 
      serde_json::ser::to_string(&resource_init_params).unwrap()
    ).as_bytes().to_vec();
    // ResourceId is only the subaccount. resource_account_id is the full near qualified name.

    let resource_account_id =
      AccountId::from_str(&*format!("{}.{}", resource_init_params.id, env::current_account_id()))
        .unwrap();

    Promise::new(resource_account_id.clone())
      .create_account()
      .transfer(env::attached_deposit()) 
      .add_full_access_key(env::signer_account_pk()) // TODO maybe use predecessor_account_key instead - but not sure how
      .deploy_contract(include_bytes!("../../target/wasm32-unknown-unknown/release/chershare_resource.wasm").to_vec())
      .function_call("new".to_string(), init_args, 0, CREATE_RESOURCE_GAS)
      .then(
        Self::ext(env::current_account_id())
          .with_static_gas(tgas(5))
          .create_resource_callback()
      )
  }

  #[private] 
  pub fn create_resource_callback(
    &mut self, 
    #[callback_result] call_result: Result<String, PromiseError>) -> () {
      match call_result {
        Ok(_string) => {
          self.resources.insert(&env::signer_account_id().to_string());
        }, 
        Err(_err) => {
        }
      }
  }
}

