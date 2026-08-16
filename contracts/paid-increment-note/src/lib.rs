#![no_std]
#![feature(alloc_error_handler)]

use miden::*;
use crate::bindings::miden::paid_counter_account::paid_counter;

#[account(paid_counter_account::PaidCounter)]
pub struct CounterAccount;

#[note]
struct PaidIncrementNote;

#[note]
impl PaidIncrementNote {
    #[note_script]
    fn run(self, _arg: Word, account: &mut CounterAccount) {
        let assets = active_note::get_assets();
        let mut processed = felt!(0);

        for asset in assets {
            assert_eq(processed, felt!(0));
            account.pay_and_increment(asset);
            processed = processed + felt!(1);
        }

        assert_eq(processed, felt!(1));
    }
}
