use bytes::Bytes;

/// Transaction format accepted by the EVM.
///
/// This isn't the format of a transaction that is given to JSONRPC but rather the format we will
/// take with the VM. For now, it will closely follow the testing format.
///
/// This is intentionally incomplete and we will slowly expand it as we require the fields.
#[derive(Debug)]
pub struct Transaction {
    pub code: Bytes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_construct_transaction() {
        let _ = Transaction {
            code: Bytes::from("hello world"),
        };
    }
}
