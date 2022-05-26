use bytes::Bytes;
use evm_rust::{Transaction, Vm};

#[test]
fn add0() {
    let vm = Vm {};
    vm.exec(Transaction {
        code: Bytes::from("asd"),
    });
}
