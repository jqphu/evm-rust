use bytes::Bytes;
use ethereum_types::H256;
use evm_rust::{Transaction, Vm};
use std::collections::HashMap;

#[test]
fn empty() {
    let mut storage = HashMap::<H256, H256>::new();

    let vm = Vm::new(&mut storage);
    vm.exec(Transaction { code: Bytes::new() })
        .expect("Should succeed and do nothing");
}

#[test]
fn stop() {
    let mut storage = HashMap::<H256, H256>::new();
    let vm = Vm::new(&mut storage);

    vm.exec(Transaction {
        code: Bytes::from_static(&[0x00]),
    })
    .expect("Should succeed and do nothing");
}
