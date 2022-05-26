use crate::Transaction;

/// EVM Implementation
pub struct Vm {}

impl Vm {
    pub fn exec(&self, transaction: Transaction) {
        println!("Executing {:?}", transaction);
    }
}
