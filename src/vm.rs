/// EVM Implementation
pub struct Vm {}

impl Vm {
    pub fn exec(&self) {
        println!("Executing");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm() {
        let vm = Vm {};
        vm.exec();
    }
}
