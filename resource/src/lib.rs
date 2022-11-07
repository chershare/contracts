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


#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct SimpleRent {
  price_per_ms: u128
}

impl SimpleRent {
  pub fn get_price(&self, from: i64, until:i64) -> u128 {
    return ((until - from) as u128) * self.price_per_ms; 
  }
  pub fn get_refund_amount(&self, from: i64, until:i64, _now: i64) -> u128 {
    return ((until - from) as u128) * self.price_per_ms; 
  }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct LinearRefund {
  price_fixed_base: u128,
  price_per_ms: u128,
  refund_buffer: i64,
}

impl LinearRefund {
  pub fn get_price(&self, from: i64, until:i64) -> u128 {
    return self.price_fixed_base + ((until - from) as u128) * self.price_per_ms; 
  }
  pub fn get_refund_amount(&self, from: i64, until:i64, now: i64) -> u128 {
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
  pub coordinates: [f32; 2], 
  pub min_duration_ms: i64,
  pub image_urls: Vec<String>, 
  pub tags: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Booking {
  begin: i64, 
  end: i64, 
  consumer_account_id: String
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Resource {
  title: String, 
  description: String, 
  pricing: PricingEnum, 
  min_duration_ms: i64, 
  contact: String, 
  image_urls: LookupSet<String>, 
  tags: LookupSet<String>, 
  next_booking_id: u128,
  blocker_beginnings: TreeMap<i64, u128>, 
  blocker_ends: TreeMap<i64, u128>, 
  bookings: LookupMap<u128, Booking>, 
  coordinates: [f32; 2]
}

#[near_bindgen]
impl Resource {
  #[init]
  #[allow(dead_code)]
  fn new(
    title: String, 
    description: String, 
    pricing: PricingEnum, 
    min_duration_ms: i64,
    contact: String, 
    image_urls: Vec<String>, 
    tags: Vec<String>, 
    coordinates: [f32; 2],
  ) -> Self {
    let mut new_resource = Self {
      title, 
      description, 
      pricing,
      min_duration_ms, 
      contact, 
      image_urls: LookupSet::new(b"i"), 
      tags: LookupSet::new(b"t"), 
      next_booking_id: 1, // 0 is reserved for resource owner blockers
      blocker_beginnings: TreeMap::new(b"b"), 
      blocker_ends: TreeMap::new(b"e"), 
      bookings: LookupMap::new(b"k"),
      coordinates,
    };
    new_resource.image_urls.extend(image_urls);
    new_resource.tags.extend(tags); 
    new_resource
  }

  pub fn assert_no_booking_collision(&self, begin: i64, end: i64) {
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
  pub fn book(&mut self, begin: i64, end: i64) {
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
