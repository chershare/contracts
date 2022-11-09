use near_sdk::json_types::U128;
use near_sdk::{env, PanicOnDefault};

use near_sdk::collections::{
  LookupSet, 
  TreeMap, 
  LookupMap 
};
use near_sdk::serde::{
    Deserialize,
    Serialize,
};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;

#[derive(Deserialize, Serialize)]
pub struct SimpleRentInitParams {
  price_per_ms: U128
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SimpleRent {
  price_per_ms: u128
}

impl SimpleRent {
  pub fn new(init_params: SimpleRentInitParams) -> Self {
    return Self {
      price_per_ms: init_params.price_per_ms.0
    }
  }

  pub fn get_price(&self, from: u64, until:u64) -> u128 {
    return ((until - from) as u128) * self.price_per_ms; 
  }
  pub fn get_refund_amount(&self, from: u64, until:u64, _now: u64) -> u128 {
    return ((until - from) as u128) * self.price_per_ms; 
  }
}

#[derive(Deserialize, Serialize)]
pub struct LinearRefundInitParams {
  price_fixed_base: U128,
  price_per_ms: U128,
  refund_buffer: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct LinearRefund {
  price_fixed_base: u128,
  price_per_ms: u128,
  refund_buffer: u64,
}

impl LinearRefund {
  pub fn new(init_params: LinearRefundInitParams) -> Self {
    return Self {
      price_fixed_base: init_params.price_fixed_base.0, 
      price_per_ms: init_params.price_per_ms.0, 
      refund_buffer: init_params.refund_buffer
    }
  }

  pub fn get_price(&self, from: u64, until:u64) -> u128 {
    return self.price_fixed_base + ((until - from) as u128) * self.price_per_ms; 
  }
  pub fn get_refund_amount(&self, from: u64, until:u64, now: u64) -> u128 {
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

#[derive(Deserialize, Serialize)]
pub enum PricingEnumInitParams {
  SimpleRent(SimpleRentInitParams), 
  LinearRefund(LinearRefundInitParams), 
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum PricingEnum {
  SimpleRent(SimpleRent), 
  LinearRefund(LinearRefund),
}

#[derive(Deserialize, Serialize)]
pub struct ResourceInitParams {
  pub title: String, 
  pub description: String, 
  pub pricing: PricingEnumInitParams, 
  pub contact: String, 
  pub coordinates: [f32; 2], 
  pub min_duration_ms: u64,
  pub image_urls: Vec<String>, 
  pub tags: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Booking {
  begin: u64, 
  end: u64, 
  consumer_account_id: String
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Resource {
  title: String, 
  description: String, 
  pricing: PricingEnum, 
  min_duration_ms: u64, 
  contact: String, 
  image_urls: LookupSet<String>, 
  tags: LookupSet<String>, 
  next_booking_id: u128,
  blocker_beginnings: TreeMap<u64, u128>, 
  blocker_ends: TreeMap<u64, u128>, 
  bookings: LookupMap<u128, Booking>, 
  coordinates: [f32; 2], 
}

#[near_bindgen]
impl Resource {
  #[init]
  pub fn init(init_params: ResourceInitParams) -> Self {
    let pricing = match init_params.pricing {
      PricingEnumInitParams::SimpleRent(ip) => {
        PricingEnum::SimpleRent(SimpleRent::new(ip))
      },
      PricingEnumInitParams::LinearRefund(ip) => {
        PricingEnum::LinearRefund(LinearRefund::new(ip))
      }
    };
    let mut resource = Self {
      title: init_params.title, 
      description: init_params.description, 
      pricing, 
      contact: init_params.contact, 
      image_urls: LookupSet::new(b"i"), 
      tags: LookupSet::new(b"t"), 
      blocker_beginnings: TreeMap::new(b"b"), 
      blocker_ends: TreeMap::new(b"e"), 
      bookings: LookupMap::new(b"k"),
      coordinates: init_params.coordinates, 
      min_duration_ms: init_params.min_duration_ms, 
      next_booking_id: 0
    };
    resource.image_urls.extend(init_params.image_urls);
    resource.tags.extend(init_params.tags); 
    resource
  }

  pub fn test() -> String {
    return "hi, cool!".into(); 
  }

  pub fn assert_no_booking_collision(&self, begin: u64, end: u64) {
    if let Some(booking_right_begin) = self.blocker_ends.higher(&begin) { // find out booking with the next end marker right of from
      if let Some(booking_right) = self.blocker_ends.get(&booking_right_begin) {
        if let Some(booking) = self.bookings.get(&booking_right) {
          assert!( // check that that one's start is after this ones end
            booking.begin > end, 
            "booking collision"
          );
        }
      }
    }
    if let Some(booking_left_begin) = self.blocker_beginnings.lower(&end) {
      if let Some(booking_left) = self.blocker_beginnings.get(&booking_left_begin) {
        if let Some(booking) = self.bookings.get(&booking_left) {
          assert!(
            booking.end < begin,
            "booking collision"
          );
        }
      }
    }
  }

  #[payable]
  pub fn book(&mut self, begin: u64, end: u64) {
    assert!(end > begin, "end before begin"); 
    let duration = end - begin;
    assert!(duration >= self.min_duration_ms);
    self.assert_no_booking_collision(begin, end); 
    let price = match &self.pricing {
      PricingEnum::SimpleRent(sr) => {
        sr.get_price(begin, end)
      }, 
      PricingEnum::LinearRefund(sr) => {
        sr.get_price(begin, end)
      }
    }; 
    assert!(
        env::attached_deposit() >= price,
        "price: {}, sent: {}",
        price,
        env::attached_deposit()
    );
    let booking_id = self.next_booking_id; 
    self.next_booking_id += 1; 
    let booking = Booking {
      begin, 
      end, 
      consumer_account_id: env::signer_account_id().to_string()
    }; 
    self.bookings.insert(&booking_id, &booking);
    self.blocker_beginnings.insert(&begin, &booking_id);
    self.blocker_ends.insert(&end, &booking_id); 
    // from the start, find the next end
  }

  //TODO fn replace_booking for changes to the booking - such that noone can interfere

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
