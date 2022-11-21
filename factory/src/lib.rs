use std::str::FromStr;

use near_sdk::borsh::{
  self,
  BorshDeserialize,
  BorshSerialize,
};
use near_sdk::collections::LookupSet;
use near_sdk::{
  self,
  env,
  near_bindgen,
  AccountId,
  Gas,
  Promise, 
  PromiseError, 
};

use chershare_resource::ResourceInitParams;
use serde::{Deserialize, Serialize};

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
  pub test_msg: String, 
}

impl Default for ChershareResourceFactory {
  fn default() -> ChershareResourceFactory {
    ChershareResourceFactory {
      resources: LookupSet::new(b"t".to_vec()),
      test_msg: "hi!".into(), 
    }
  }
}

#[derive(Deserialize, Serialize)]
struct ResourceInitParamsCallWrapper {
  owner: String, 
  init_params: ResourceInitParams
}

#[derive(Deserialize, Serialize)]
struct ResourceCreationLog {
  name: String, 
  owner: String, 
  init_params: ResourceInitParams, 
}

#[near_bindgen]
impl ChershareResourceFactory {
  pub fn check_resource_contained(
    &self,
    resource_id: &String,
  ) -> bool {
    self.resources.contains(resource_id)
  }

  pub fn assert_name_available(
    &self,
    resource_id: &String,
  ) {
    assert!(
      !self.check_resource_contained(resource_id),
      "Resource with that ID already exists"
    );
  }

  pub fn get_test(&self) -> String {
    self.test_msg.clone()
  }

  pub fn name_exists(&self, name: String) -> bool {
    self.resources.contains(&name)
  }

  #[payable]
  pub fn create_resource(
    &mut self,
    name: String,
    resource_init_params: ResourceInitParams 
  ) -> Promise {
    self.assert_name_available(&name);

    let resource_owner = env::signer_account_id(); 

    // prepare arguments as json byte vector
    let init_args = serde_json::ser::to_string(&ResourceInitParamsCallWrapper {
      owner: resource_owner.to_string(), 
      init_params: resource_init_params.clone(), 
    }).unwrap().as_bytes().to_vec();

    // ResourceId is only the subaccount. resource_account_id is the full near qualified name.
    let resource_account_id =
      AccountId::from_str(&*format!("{}.{}", name, env::current_account_id()))
        .unwrap();

    Promise::new(resource_account_id.clone())
      .create_account()
      .transfer(env::attached_deposit()) 
      .add_full_access_key(env::signer_account_pk()) // TODO maybe use predecessor_account_key instead - but not sure how
      .deploy_contract(include_bytes!("../../target/wasm32-unknown-unknown/release/chershare_resource.wasm").to_vec())
      .function_call("init".to_string(), init_args, 0, CREATE_RESOURCE_GAS)
      .then(
        Self::ext(env::current_account_id())
          .with_static_gas(tgas(10))
          .create_resource_callback(name, resource_owner.to_string(), resource_init_params)
      )
  }

  #[private] 
  pub fn create_resource_callback(
    &mut self, 
    name: String,
    owner: String, 
    init_params: ResourceInitParams, 
    #[callback_result] call_result: Result<(), PromiseError>) -> () {
      match call_result {
        // TODO: indexer should only record succesful resource creations
        Ok(_string) => {
          self.resources.insert(&name);// &env::signer_account_id().to_string());
          env::log_str(
            &*format!("ResourceCreation: {}", serde_json::ser::to_string(&ResourceCreationLog {
              name, 
              owner, 
              init_params, 
            }).unwrap())
          ); 
        }, 
        Err(_err) => {
        }
      }
  }
}

