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
struct BookingCreationLog {
  id: U128,
  booker_account_id: String, 
  start: u64, 
  end: u64, 
  price: U128
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PricingParams {
  price_per_ms: U128,
  price_per_booking: U128,
  full_refund_period_ms: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pricing {
  price_fixed_base: u128,
  price_per_ms: u128,
  refund_buffer: u64,
}

impl Pricing {
  pub fn new(init_params: PricingParams) -> Self {
    return Self {
      price_fixed_base: init_params.price_per_booking.0, 
      price_per_ms: init_params.price_per_ms.0, 
      refund_buffer: init_params.full_refund_period_ms
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

#[derive(Deserialize, Serialize, Clone)]
pub struct ResourceInitParams {
  pub title: String, 
  pub description: String, 
  pub image_urls: Vec<String>, 
  pub contact: String, 
  pub tags: Vec<String>,
  pub pricing: PricingParams,  
  pub coordinates: [f32; 2], 
  pub min_duration_ms: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Booking {
  start: u64, 
  end: u64, 
  consumer_account_id: String
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Resource {
  title: String, 
  description: String, 
  pricing: Pricing, 
  min_duration_ms: u64, 
  contact: String, 
  image_urls: LookupSet<String>, 
  tags: LookupSet<String>, 
  next_booking_id: u128,
  blocker_starts: TreeMap<u64, u128>, 
  blocker_ends: TreeMap<u64, u128>, 
  bookings: LookupMap<u128, Booking>, 
  coordinates: [f32; 2], 
}

#[near_bindgen]
impl Resource {
  #[init]
  pub fn init(init_params: ResourceInitParams) -> Self {
    let pricing = Pricing::new(init_params.pricing);
    let mut resource = Self {
      title: init_params.title, 
      description: init_params.description, 
      pricing, 
      contact: init_params.contact, 
      image_urls: LookupSet::new(b"i"), 
      tags: LookupSet::new(b"t"), 
      blocker_starts: TreeMap::new(b"b"), 
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

  pub fn assert_no_booking_collision(&self, start: u64, end: u64) {
    if let Some(booking_right_start) = self.blocker_ends.higher(&start) { // find out booking with the next end marker right of from
      if let Some(booking_right) = self.blocker_ends.get(&booking_right_start) {
        if let Some(booking) = self.bookings.get(&booking_right) {
          assert!( // check that that one's start is after this ones end
            booking.start > end, 
            "booking collision"
          );
        }
      }
    }
    if let Some(booking_left_start) = self.blocker_starts.lower(&end) {
      if let Some(booking_left) = self.blocker_starts.get(&booking_left_start) {
        if let Some(booking) = self.bookings.get(&booking_left) {
          assert!(
            booking.end < start,
            "booking collision"
          );
        }
      }
    }
  }

  #[payable]
  pub fn book(&mut self, start: u64, end: u64) {
    assert!(end > start, "end before start"); 
    let duration = end - start;
    assert!(duration >= self.min_duration_ms);
    self.assert_no_booking_collision(start, end); 
    let price = self.pricing.get_price(start, end);
    assert!(
        env::attached_deposit() >= price,
        "price: {}, sent: {}",
        price,
        env::attached_deposit()
    );
    let booking_id = self.next_booking_id; 
    self.next_booking_id += 1; 
    let booking = Booking {
      start, 
      end, 
      consumer_account_id: env::signer_account_id().to_string()
    }; 
    self.bookings.insert(&booking_id, &booking);
    self.blocker_starts.insert(&start, &booking_id);
    self.blocker_ends.insert(&end, &booking_id); 

    env::log_str(&*format!("BookingCreation: {}", serde_json::ser::to_string(&BookingCreationLog {
      id: U128::from(booking_id),
      booker_account_id: booking.consumer_account_id, 
      start: booking.start, 
      end: booking.end, 
      price: U128::from(price) 
    }).unwrap())); 
    // from the start, find the next end
  }

  pub fn get_quote(&self, start: u64, end: u64) -> U128 {
    U128::from(self.pricing.get_price(start, end))
  }
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
