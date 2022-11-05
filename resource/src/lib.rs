/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use near_sdk::serde::{
    Deserialize,
    Serialize,
};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;

pub trait Pricing {
  fn get_price(&self, from: i64, until: i64) -> i128; 
  fn get_refund_amount(&self, from: i64, until: i64, now: i64) -> i128; 
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SimpleRent {
  price_per_ms: i128
}

impl Pricing for SimpleRent {
  fn get_price(&self, from: i64, until:i64) -> i128 {
    return ((until - from) as i128) * self.price_per_ms; 
  }
  fn get_refund_amount(&self, from: i64, until:i64, _now: i64) -> i128 {
    return ((until - from) as i128) * self.price_per_ms; 
  }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Rent {
  price_fixed_base: i128,
  price_per_ms: i128,
  refund_buffer: i64,
}

impl Pricing for Rent {
  fn get_price(&self, from: i64, until:i64) -> i128 {
    return self.price_fixed_base + ((until - from) as i128) * self.price_per_ms; 
  }
  fn get_refund_amount(&self, from: i64, until:i64, now: i64) -> i128 {
    let price_payed = self.get_price(from, until);
    if now < from {
      let distance = from - now; 
      if distance < self.refund_buffer { // TODO: consider linear
        const PRECISION: i128 = 1000;
        let squared_progress = i128::from(self.refund_buffer - distance).pow(2);
        let squared_refund_buffer = i128::from(self.refund_buffer).pow(2);
        let factor = PRECISION * (squared_refund_buffer - squared_progress) / squared_refund_buffer; 
        price_payed * factor / PRECISION
      } else {
        price_payed
      }
    } else {
      0 
    }
  } // fees will not be payed back due to technical reasons
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Resource {
  id: String, 
  title: String, 
  description: String, 
  pricing: SimpleRent, 
  contact: String, 
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct ResourceInitParams {
  pub id: String, 
  pub title: String, 
  pub description: String, 
  pub price_per_ms: i128, 
  pub contact: String, 
}

#[near_bindgen]
impl Resource {
  #[init]
  fn new(
    id: String, 
    title: String, 
    description: String, 
    price_per_ms: i128, 
    contact: String, 
  ) -> Self {
    Self {
      id, 
      title, 
      description, 
      pricing: SimpleRent {
        price_per_ms
      }, 
      contact, 
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
