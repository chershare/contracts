use near_sdk::serde::{
    Deserialize,
    Serialize,
};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;

pub trait Pricing {
  fn get_price(&self, from: i64, until: i64) -> u128; 
  fn get_refund_amount(&self, from: i64, until: i64, now: i64) -> u128; 
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct SimpleRent {
  price_per_ms: u128
}

impl Pricing for SimpleRent {
  fn get_price(&self, from: i64, until:i64) -> u128 {
    return ((until - from) as u128) * self.price_per_ms; 
  }
  fn get_refund_amount(&self, from: i64, until:i64, _now: i64) -> u128 {
    return ((until - from) as u128) * self.price_per_ms; 
  }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct LinearRefund {
  price_fixed_base: u128,
  price_per_ms: u128,
  refund_buffer: i64,
}

impl Pricing for LinearRefund {
  fn get_price(&self, from: i64, until:i64) -> u128 {
    return self.price_fixed_base + ((until - from) as u128) * self.price_per_ms; 
  }
  fn get_refund_amount(&self, from: i64, until:i64, now: i64) -> u128 {
    let price_payed = self.get_price(from, until);
    if now < from {
      let distance = from - now; 
      if distance < self.refund_buffer { 
        price_payed * distance as u128 / self.refund_buffer as u128
      } else {
        price_payed
      }
    } else {
      0 
    }
  } // fees will not be payed back due to technical reasons
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub enum PricingEnum {
  SimpleRent(SimpleRent), 
  LinearRefund(LinearRefund),
}

#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "wasm", derive(BorshDeserialize, BorshSerialize))]
pub struct ResourceInitParams {
  pub title: String, 
  pub description: String, 
  pub pricing: PricingEnum, 
  pub contact: String, 
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Resource {
  title: String, 
  description: String, 
  pricing: PricingEnum, 
  contact: String, 
}

#[near_bindgen]
impl Resource {
  #[init]
  #[allow(dead_code)]
  fn new(
    title: String, 
    description: String, 
    pricing: PricingEnum, 
    contact: String, 
  ) -> Self {
    Self {
      title, 
      description, 
      pricing,
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
