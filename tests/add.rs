use bytes::Bytes;
use ethereum_types::H256;
use evm_rust::{Transaction, Vm};
use hex_literal::hex;
use std::collections::HashMap;

#[test]
fn add0() {
    let mut storage = HashMap::<H256, H256>::new();
    let vm = Vm::new(&mut storage);
    vm.exec(Transaction {
        code: Bytes::from_static(&hex!("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0160005500")),
    }).unwrap();

    assert_eq!(
        storage.get(&H256::zero()).unwrap(),
        &H256::from(hex!(
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"
        ))
    );
}

#[test]
fn add1() {
    let mut storage = HashMap::<H256, H256>::new();
    let vm = Vm::new(&mut storage);
    vm.exec(Transaction {
        code: Bytes::from_static(&hex!(
            "60047fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0160005500"
        )),
    })
    .unwrap();

    assert_eq!(
        storage.get(&H256::zero()).unwrap(),
        &H256::from(hex!(
            "0000000000000000000000000000000000000000000000000000000000000003"
        ))
    );
}
