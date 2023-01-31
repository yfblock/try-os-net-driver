/// When the program uses this crate, it needs to implement this trait.
pub trait NetWorkStackOp {
    fn write(data: &[u8]);
    fn read(data: &mut [u8]);
}

/// call this function when the eth interrupt was occured.
fn handle_interrupt() {
    todo!("The real handler is not being called for now.");
}