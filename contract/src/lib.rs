/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
  log, 
  near_bindgen, 
  Promise, 
};

use near_sdk::env::panic_str; 

use near_sdk::collections::LookupMap;  


pub trait Pricing {
  fn get_price(&self, from: i64, until: i64) -> i128; 
  fn get_refund(&self, from: i64, until: i64, now: i64) -> i128; 
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SimpleRent {
  price_per_ms: i128
}

impl Pricing for SimpleRent {
  fn get_price(&self, from: i64, until:i64) -> i128 {
    return ((until - from) as i128) * self.price_per_ms; 
  }
  fn get_refund(&self, from: i64, until:i64, now: i64) -> i128 {
    return ((until - from) as i128) * self.price_per_ms; 
  }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Resource {
  name: String, 
  description: String, 
  pricing: SimpleRent 
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
  resources: LookupMap<String, Resource>
}

// Define the default, which automatically initializes the contract
impl Default for Contract{
  fn default() -> Self{
    Self{
      resources: LookupMap::new(b"r".to_vec())
    }
  }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
  pub fn create_resource (
    &mut self, 
    id: String, 
    name: String, 
    description: String, 
    price_per_ms: i128 
  ) {
    match self.resources.get(&id) {
      Some(..) => {
        panic_str("A resource with this id already exists")
      }, 
      None => {
        self.resources.insert(&id, {
          &Resource {
            name, 
            description, 
            pricing: SimpleRent {
              price_per_ms
            }
          }
        });
      }
    }
  }

  //TODO get resource - überhaupt nötig? weil eigentlich wollen wir ja über einen Indexer. 

  // Example Methods
  // // Public method - returns the greeting saved, defaulting to DEFAULT_MESSAGE
  // pub fn get_greeting(&self) -> String {
  //     return self.message.clone();
  // }

  // // Public method - accepts a greeting, such as "howdy", and records it
  // pub fn set_greeting(&mut self, message: String) {
  //     // Use env::log to record logs permanently to the blockchain!
  //     log!("Saving greeting {}", message);
  //     self.message = message;
  // }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
  // Example tests

  // use super::*;

  // #[test]
  // fn get_default_greeting() {
  //     let contract = Contract::default();
  //     // this test did not call set_greeting so should return the default "Hello" greeting
  //     assert_eq!(
  //         contract.get_greeting(),
  //         "Hello".to_string()
  //     );
  // }

  // #[test]
  // fn set_then_get_greeting() {
  //     let mut contract = Contract::default();
  //     contract.set_greeting("howdy".to_string());
  //     assert_eq!(
  //         contract.get_greeting(),
  //         "howdy".to_string()
  //     );
  // }
}
