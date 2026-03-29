use common::error::NovaResult;
use rand::Rng;
use vm::memory_manager::VmData;
use vm::state;

pub fn random_int(state: &mut state::State) -> NovaResult<()> {
    if let (Some(VmData::Int(high)), Some(VmData::Int(low))) =
        (state.memory.stack.pop(), state.memory.stack.pop())
    {
        let mut rng = rand::thread_rng();
        state
            .memory
            .stack
            .push(VmData::Int(rng.gen_range(low..=high)));
    }
    Ok(())
}
