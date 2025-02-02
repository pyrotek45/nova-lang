use std::{
    cell::{RefCell, RefMut},
    io::Write,
    process::exit,
    rc::Rc,
};

use common::error::NovaError;
use rand::Rng;
use raylib::prelude::*;
use vm::state::{self, Draw, Heap, VmData};

use crate::time::sleep;

// function each time the raylib is called to check if the window is closed, but doesnt push anything to the stack or exits if it is closed
pub fn raylib_check_window(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(rl) = state.raylib.as_ref() {
        let rl = state.raylib.as_ref().unwrap().borrow_mut();
        let window_should_close = rl.window_should_close();
        if window_should_close {
            exit(0);
        }
    }
    Ok(())
}

pub fn raylib_init(state: &mut state::State) -> Result<(), NovaError> {
    let fps = state.stack.pop().unwrap();
    let h = state.stack.pop().unwrap();
    let w = state.stack.pop().unwrap();
    let text = match state.stack.pop() {
        Some(VmData::String(index)) => match state.get_ref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };
    let (w, h) = match (w, h) {
        (VmData::Int(x), VmData::Int(y)) => (x, y),
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    let fps = match fps {
        VmData::Int(fps) => fps,
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    let (mut rl, thread) = raylib::init().size(w as i32, h as i32).title(&text).build();
    rl.set_target_fps(fps as u32);
    state.raylib = Some(Rc::new(RefCell::new(rl)));
    state.raylib_thread = Some(thread);

    std::io::stdout().flush().unwrap();
    let mut rl = state.raylib.as_ref().unwrap().borrow_mut();
    let thread = state.raylib_thread.as_ref().unwrap();

    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::WHITE);

    Ok(())
}

pub fn raylib_rendering(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(rl) = state.raylib.as_ref() {
        let thread = state.raylib_thread.as_ref().unwrap().clone();
        let mut rl = state.raylib.as_ref().unwrap().borrow_mut();

        let window_should_close = rl.window_should_close();
        let mut d = rl.begin_drawing(&thread);

        // use state draw_queue to draw all the items while emptying the queue
        for draw in state.draw_queue.drain(..) {
            match draw {
                Draw::Text {
                    x,
                    y,
                    text,
                    size,
                    color,
                } => {
                    // will use default color since its prototype
                    d.draw_text(&text, x as i32, y as i32, size as i32, color);
                }
                Draw::FPS { x, y } => {
                    d.draw_fps(x as i32, y as i32);
                }
                Draw::Rectangle {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    d.draw_rectangle(x as i32, y as i32, width as i32, height as i32, color);
                }
                Draw::Circle {
                    x,
                    y,
                    radius,
                    color,
                } => todo!(),
                Draw::Line {
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                    color,
                } => todo!(),
                Draw::ClearBackground { color } => {
                    d.clear_background(color);
                }
            }
        }

        // return false if window is closed
        if window_should_close {
            state.stack.push(VmData::Bool(false));
        } else {
            state.stack.push(VmData::Bool(true));
        }
    }
    Ok(())
}

// raylib sleep function
pub fn raylib_sleep(state: &mut state::State) -> Result<(), NovaError> {
    let ms = state.stack.pop().unwrap();
    let ms = match ms {
        VmData::Int(ms) => ms,
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    let thread = state.raylib_thread.as_ref().unwrap();
    let mut rl: RefMut<RaylibHandle> = state.raylib.as_ref().unwrap().borrow_mut();
    rl.wait_time(ms as f64);
    Ok(())
}

// draw text simple hello world
pub fn raylib_draw_text(state: &mut state::State) -> Result<(), NovaError> {
    // get x and y
    // raylib_check_window(state)?;
    // get color as tuple
    let color = state.stack.pop().unwrap();
    let (r, g, b) = match color {
        VmData::List(index) => {
            // get value from heap
            let tuple = state.get_ref(index);
            match tuple {
                Heap::List(pointers) => {
                    // vec of pointers to get the values
                    let mut r = state.get_ref(pointers[0]).get_int();
                    let mut g = state.get_ref(pointers[1]).get_int();
                    let mut b = state.get_ref(pointers[2]).get_int();
                    (r, g, b)
                }
                _ => {
                    return Err(NovaError::Runtime {
                        msg: "Expected tuple".into(),
                    })
                }
            }
        }
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected tuple".into(),
            })
        }
    };
    // get size of text
    let size = state.stack.pop().unwrap();
    let size = match size {
        VmData::Int(size) => size,
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    let y = state.stack.pop().unwrap();
    let x = state.stack.pop().unwrap();
    let text = match state.stack.pop() {
        Some(VmData::String(index)) => match state.get_ref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };
    let (x, y) = match (x, y) {
        (VmData::Int(x), VmData::Int(y)) => (x, y),
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    let color = Color::new(r as u8, g as u8, b as u8, 255);
    state.draw_queue.push(Draw::Text {
        x: x as i32,
        y: y as i32,
        text: text.clone(),
        size: size as i32,
        color,
    });

    Ok(())
}

// clear the screen
pub fn raylib_clear(state: &mut state::State) -> Result<(), NovaError> {
    let color = state.stack.pop().unwrap();
    let (r, g, b) = match color {
        VmData::List(index) => {
            // get value from heap
            let tuple = state.get_ref(index);
            match tuple {
                Heap::List(pointers) => {
                    // vec of pointers to get the values
                    let mut r = state.get_ref(pointers[0]).get_int();
                    let mut g = state.get_ref(pointers[1]).get_int();
                    let mut b = state.get_ref(pointers[2]).get_int();
                    (r, g, b)
                }
                _ => {
                    return Err(NovaError::Runtime {
                        msg: "Expected tuple".into(),
                    })
                }
            }
        }
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected tuple".into(),
            })
        }
    };
    let color = Color::new(r as u8, g as u8, b as u8, 255);
    state.draw_queue.push(Draw::ClearBackground { color });
    Ok(())
}

// draw fps
pub fn raylib_draw_fps(state: &mut state::State) -> Result<(), NovaError> {
    // raylib_check_window(state)?;
    let (w, h) = match (state.stack.pop().unwrap(), state.stack.pop().unwrap()) {
        (VmData::Int(w), VmData::Int(h)) => (w, h),
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    state.draw_queue.push(Draw::FPS {
        x: w as i32,
        y: h as i32,
    });
    Ok(())
}

// raylib get mouse position -> returns tuple (x, y)
pub fn raylib_get_mouse_position(state: &mut state::State) -> Result<(), NovaError> {
    // raylib_check_window(state)?;
    let thread = state.raylib_thread.as_ref().unwrap().clone();
    let pos = {
        let rl = state.raylib.as_ref().unwrap().borrow_mut();
        rl.get_mouse_position()
    };
    // store in list and push to stack
    state.gclock = true;
    let x = state.allocate_vmdata_to_heap(VmData::Int(pos.x as i64));
    let y = state.allocate_vmdata_to_heap(VmData::Int(pos.y as i64));
    let index = state.allocate_array(vec![x, y]);
    state.stack.push(VmData::List(index));
    state.gclock = false;
    Ok(())
}

// draw rectangle
pub fn raylib_draw_rectangle(state: &mut state::State) -> Result<(), NovaError> {
    // raylib_check_window(state)?;
    let color = state.stack.pop().unwrap();
    let (r, g, b) = match color {
        VmData::List(index) => {
            // get value from heap
            let tuple = state.get_ref(index);
            match tuple {
                Heap::List(pointers) => {
                    // vec of pointers to get the values
                    let mut r = state.get_ref(pointers[0]).get_int();
                    let mut g = state.get_ref(pointers[1]).get_int();
                    let mut b = state.get_ref(pointers[2]).get_int();
                    (r, g, b)
                }
                _ => {
                    return Err(NovaError::Runtime {
                        msg: "Expected tuple".into(),
                    })
                }
            }
        }
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected tuple".into(),
            })
        }
    };
    let color = Color::new(r as u8, g as u8, b as u8, 255);
    let height = state.stack.pop().unwrap();
    let width = state.stack.pop().unwrap();
    let y = state.stack.pop().unwrap();
    let x = state.stack.pop().unwrap();
    let (x, y) = match (x, y) {
        (VmData::Int(x), VmData::Int(y)) => (x, y),
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    let width = match width {
        VmData::Int(width) => width,
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    let height = match height {
        VmData::Int(height) => height,
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected integer".into(),
            })
        }
    };
    state.draw_queue.push(Draw::Rectangle {
        x: x as i32,
        y: y as i32,
        width: width as i32,
        height: height as i32,
        color,
    });
    Ok(())
}

// get input from keyboard using raylib and return string representation of the key -> return Option<String>
pub fn raylib_get_key_as_string(state: &mut state::State) -> Result<(), NovaError> {
    // raylib_check_window(state)?;
    // use raylib to get the key pressed, not the stack, and return the string representation of the key
    let key = state
        .raylib
        .as_ref()
        .unwrap()
        .borrow_mut()
        .get_key_pressed();
    if let Some(key) = key {
        let key = Rc::from(format!("{:?}", key));
        let index = state.allocate_string(key);
        state.stack.push(VmData::String(index));
    } else {
        state.stack.push(VmData::None);
    }

    Ok(())
}

// check if key is pressed down -> return bool
pub fn raylib_is_key_down(state: &mut state::State) -> Result<(), NovaError> {
    // raylib_check_window(state)?;
    let key = state.stack.pop().unwrap();
    let key = match key {
        VmData::String(index) => {
            let key = state.get_ref(index);
            match key {
                Heap::String(key) => key,
                _ => {
                    return Err(NovaError::Runtime {
                        msg: "Expected string".into(),
                    })
                }
            }
        }
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected string".into(),
            })
        }
    };
    let key = key.to_string();
    let key = key.as_str();
    let key = match key {
        "KEY_A" => KeyboardKey::KEY_A,
        "KEY_B" => KeyboardKey::KEY_B,
        "KEY_C" => KeyboardKey::KEY_C,
        "KEY_D" => KeyboardKey::KEY_D,
        "KEY_E" => KeyboardKey::KEY_E,
        "KEY_F" => KeyboardKey::KEY_F,
        "KEY_G" => KeyboardKey::KEY_G,
        "KEY_H" => KeyboardKey::KEY_H,
        "KEY_I" => KeyboardKey::KEY_I,
        "KEY_J" => KeyboardKey::KEY_J,
        "KEY_K" => KeyboardKey::KEY_K,
        "KEY_L" => KeyboardKey::KEY_L,
        "KEY_M" => KeyboardKey::KEY_M,
        "KEY_N" => KeyboardKey::KEY_N,
        "KEY_O" => KeyboardKey::KEY_O,
        "KEY_P" => KeyboardKey::KEY_P,
        "KEY_Q" => KeyboardKey::KEY_Q,
        "KEY_R" => KeyboardKey::KEY_R,
        "KEY_S" => KeyboardKey::KEY_S,
        "KEY_T" => KeyboardKey::KEY_T,
        "KEY_U" => KeyboardKey::KEY_U,
        "KEY_V" => KeyboardKey::KEY_V,
        "KEY_W" => KeyboardKey::KEY_W,
        "KEY_X" => KeyboardKey::KEY_X,
        "KEY_Y" => KeyboardKey::KEY_Y,
        "KEY_Z" => KeyboardKey::KEY_Z,
        "KEY_ZERO" => KeyboardKey::KEY_ZERO,
        "KEY_ONE" => KeyboardKey::KEY_ONE,
        "KEY_TWO" => KeyboardKey::KEY_TWO,
        "KEY_THREE" => KeyboardKey::KEY_THREE,
        "KEY_FOUR" => KeyboardKey::KEY_FOUR,
        "KEY_FIVE" => KeyboardKey::KEY_FIVE,
        "KEY_SIX" => KeyboardKey::KEY_SIX,
        "KEY_SEVEN" => KeyboardKey::KEY_SEVEN,
        "KEY_EIGHT" => KeyboardKey::KEY_EIGHT,
        "KEY_NINE" => KeyboardKey::KEY_NINE,
        "KEY_SPACE" => KeyboardKey::KEY_SPACE,
        "KEY_ENTER" => KeyboardKey::KEY_ENTER,
        "KEY_BACKSPACE" => KeyboardKey::KEY_BACKSPACE,
        "KEY_DELETE" => KeyboardKey::KEY_DELETE,
        "KEY_TAB" => KeyboardKey::KEY_TAB,
        "KEY_ESCAPE" => KeyboardKey::KEY_ESCAPE,
        "KEY_RIGHT" => KeyboardKey::KEY_RIGHT,
        "KEY_LEFT" => KeyboardKey::KEY_LEFT,
        "KEY_UP" => KeyboardKey::KEY_UP,
        "KEY_DOWN" => KeyboardKey::KEY_DOWN,
        a => {
            return Err(NovaError::Runtime {
                msg: format!("Invalid key: {}", a).into(),
            })
        }
    };
    let is_down = state.raylib.as_ref().unwrap().borrow_mut().is_key_down(key);
    state.stack.push(VmData::Bool(is_down));
    Ok(())
}
