use bytes::Bytes;
use evm_rust::{Transaction, Vm};
use hex_literal::hex;

#[test]
fn add0() {
    let vm = Vm::default();
    vm.exec(Transaction {
        code: Bytes::from_static(&hex!("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0160005500")),
    }).unwrap();
}
