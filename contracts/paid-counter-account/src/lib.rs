#![no_std]
#![feature(alloc_error_handler)]

use miden::{component, component_storage, felt, native_account, Asset, Felt, StorageMap, Word};

#[component_storage]
struct PaidCounterStorage {
    #[storage(description = "paid counter value")]
    count_map: StorageMap<Word, Felt>,
}

#[component]
trait PaidCounter {
    fn get_count(&self) -> Felt;
    fn pay_and_increment(&mut self, payment: Asset) -> Felt;
}

#[component]
impl PaidCounter for PaidCounterStorage {
    fn get_count(&self) -> Felt {
        let key = Word::new([felt!(0), felt!(0), felt!(0), felt!(1)]);
        self.count_map.get(key)
    }

    fn pay_and_increment(&mut self, payment: Asset) -> Felt {
        let key = Word::new([felt!(0), felt!(0), felt!(0), felt!(1)]);
        let current_value = self.count_map.get(key);
        let required_payment = current_value + felt!(1);
        let payment_amount = payment.inner[0];

        assert!(
            payment_amount.as_u64() >= required_payment.as_u64(),
            "payment below required counter price"
        );

        native_account::add_asset(payment);

        let new_value = current_value + felt!(1);
        self.count_map.set(key, new_value);
        new_value
    }
}
