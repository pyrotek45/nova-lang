use common::error::NovaError;
use rand::Rng;
use vm::state::{self, VmData};

pub fn random_int(state: &mut state::State) -> Result<(), NovaError> {
    if let (Some(VmData::Int(high)), Some(VmData::Int(low))) =
        (state.stack.pop(), state.stack.pop())
    {
        let mut rng = rand::thread_rng();
        // Generate a random integer between lower_bound (inclusive) and upper_bound (exclusive)
        state.stack.push(VmData::Int(rng.gen_range(low..=high)));
    }
    Ok(())
}
