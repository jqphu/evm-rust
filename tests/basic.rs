use bytes::Bytes;
use evm_rust::{Transaction, Vm};

#[test]
fn empty() {
    let vm = Vm::default();
    vm.exec(Transaction { code: Bytes::new() })
        .expect("Should succeed and do nothing");
}

#[test]
fn stop() {
    let vm = Vm::default();
    vm.exec(Transaction {
        code: Bytes::from_static(&[0x00]),
    })
    .expect("Should succeed and do nothing");
}
