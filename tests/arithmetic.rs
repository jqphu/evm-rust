use bytes::Bytes;
use ethereum_types::H256;
use evm_rust::{Transaction, Vm};
use glob::glob;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

#[derive(Deserialize)]
struct Exec {
    code: String,
}

#[derive(Deserialize)]
struct TestCase {
    exec: Exec,
    post: Value,
}

/// All the addition tests
///
/// This is the ugliest code I have ever seen. Let's just make do with it and get through more
/// tests then we can refactor it.
#[test]
fn arithmetic() {
    for entry in glob("**/arithmetic/*.json").expect("Failed to read glob pattern") {
        let path = entry.unwrap();
        println!("running test: {}", path.display());

        let filename_without_extension = path.file_stem().unwrap();

        let TestCase { exec, post } = {
            let data = fs::read_to_string(&path).expect("Unable to read file");

            let res: Value = serde_json::from_str(&data).expect("Unable to parse");

            let test_case = &res[&filename_without_extension.to_str().unwrap()];

            serde_json::from_value(test_case.clone()).unwrap()
        };

        let code = Bytes::from(hex::decode(exec.code.split_at(2).1).unwrap());

        let expected_storage: HashMap<String, String> = {
            serde_json::from_value(
                post["0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6"]["storage"].clone(),
            )
            .unwrap()
        };

        let mut storage = HashMap::<H256, H256>::new();
        let vm = Vm::new(&mut storage);
        vm.exec(Transaction { code }).unwrap();

        for (key, value) in expected_storage {
            let key = H256::from_str(&format!("{:0>64}", key.split_at(2).1)).unwrap();
            let value = H256::from_str(&format!("{:0>64}", value.split_at(2).1)).unwrap();

            assert_eq!(storage.get(&key).unwrap(), &value);
        }
    }
}
