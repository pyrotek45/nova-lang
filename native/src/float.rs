use common::error::{NovaError, NovaResult};
use vm::memory_manager::VmData;
use vm::state;

fn runtime_err(msg: impl Into<std::borrow::Cow<'static, str>>) -> Box<NovaError> {
    Box::new(NovaError::Runtime { msg: msg.into() })
}

fn pop(state: &mut state::State) -> NovaResult<VmData> {
    state
        .memory
        .stack
        .pop()
        .ok_or(runtime_err("Stack is empty"))
}

fn pop_float(state: &mut state::State) -> NovaResult<f64> {
    match pop(state)? {
        VmData::Float(v) => Ok(v),
        VmData::Int(v) => Ok(v as f64),
        _ => Err(runtime_err("Expected a Float on the stack")),
    }
}

/// Cast::float(v) -> Option(Float)
pub fn int_to_float(state: &mut state::State) -> NovaResult<()> {
    let data = pop(state)?;
    let float = match data {
        VmData::Int(v) => v as f64,
        VmData::Float(v) => v,
        VmData::Bool(v) => {
            if v {
                1.0
            } else {
                0.0
            }
        }
        VmData::Char(v) => match v.to_string().parse::<f64>() {
            Ok(n) => n,
            Err(_) => {
                state.memory.stack.push(VmData::None);
                return Ok(());
            }
        },
        VmData::Object(v) => {
            if let Some(obj) = state.memory.ref_from_heap(v) {
                if let Some(s) = obj.as_string() {
                    match s.parse::<f64>() {
                        Ok(n) => n,
                        Err(_) => {
                            state.memory.stack.push(VmData::None);
                            return Ok(());
                        }
                    }
                } else {
                    state.memory.stack.push(VmData::None);
                    return Ok(());
                }
            } else {
                state.memory.stack.push(VmData::None);
                return Ok(());
            }
        }
        _ => {
            state.memory.stack.push(VmData::None);
            return Ok(());
        }
    };
    state.memory.stack.push(VmData::Float(float));
    Ok(())
}

/// Float::floor(x) -> Float
pub fn float_floor(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.floor()));
    Ok(())
}

/// Float::ceil(x) -> Float
pub fn float_ceil(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.ceil()));
    Ok(())
}

/// Float::round(x) -> Float
pub fn float_round(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.round()));
    Ok(())
}

/// Float::abs(x) -> Float
pub fn float_abs(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.abs()));
    Ok(())
}

/// Float::sqrt(x) -> Float
pub fn float_sqrt(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.sqrt()));
    Ok(())
}

/// Float::sin(x) -> Float
pub fn float_sin(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.sin()));
    Ok(())
}

/// Float::cos(x) -> Float
pub fn float_cos(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.cos()));
    Ok(())
}

/// Float::tan(x) -> Float
pub fn float_tan(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.tan()));
    Ok(())
}

/// Float::atan2(y, x) -> Float
pub fn float_atan2(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    let y = pop_float(state)?;
    state.memory.stack.push(VmData::Float(y.atan2(x)));
    Ok(())
}

/// Float::log(x) -> Float  (natural log)
pub fn float_log(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.ln()));
    Ok(())
}

/// Float::log10(x) -> Float
pub fn float_log10(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.log10()));
    Ok(())
}

/// Float::pow(base, exp) -> Float
pub fn float_pow(state: &mut state::State) -> NovaResult<()> {
    let exp = pop_float(state)?;
    let base = pop_float(state)?;
    state.memory.stack.push(VmData::Float(base.powf(exp)));
    Ok(())
}

/// Float::exp(x) -> Float  (e^x)
pub fn float_exp(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.exp()));
    Ok(())
}

/// Float::min(a, b) -> Float
pub fn float_min(state: &mut state::State) -> NovaResult<()> {
    let b = pop_float(state)?;
    let a = pop_float(state)?;
    state.memory.stack.push(VmData::Float(a.min(b)));
    Ok(())
}

/// Float::max(a, b) -> Float
pub fn float_max(state: &mut state::State) -> NovaResult<()> {
    let b = pop_float(state)?;
    let a = pop_float(state)?;
    state.memory.stack.push(VmData::Float(a.max(b)));
    Ok(())
}

/// Float::clamp(x, min, max) -> Float
pub fn float_clamp(state: &mut state::State) -> NovaResult<()> {
    let max = pop_float(state)?;
    let min = pop_float(state)?;
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Float(x.clamp(min, max)));
    Ok(())
}

/// Float::isNan(x) -> Bool
pub fn float_is_nan(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Bool(x.is_nan()));
    Ok(())
}

/// Float::isInfinite(x) -> Bool
pub fn float_is_infinite(state: &mut state::State) -> NovaResult<()> {
    let x = pop_float(state)?;
    state.memory.stack.push(VmData::Bool(x.is_infinite()));
    Ok(())
}

/// Float::PI constant
pub fn float_pi(state: &mut state::State) -> NovaResult<()> {
    state.memory.stack.push(VmData::Float(std::f64::consts::PI));
    Ok(())
}

/// Float::E constant
pub fn float_e(state: &mut state::State) -> NovaResult<()> {
    state.memory.stack.push(VmData::Float(std::f64::consts::E));
    Ok(())
}
