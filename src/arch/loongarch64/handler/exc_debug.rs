use super::TrapFrame;

pub fn handler(tf: &mut TrapFrame, token: usize) {
    // tf always equal to 0
    // noting need to be implemented?
    log::trace!("Debug exception occurred");
}
