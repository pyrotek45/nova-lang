use std::{cell::RefCell, rc::Rc};

use common::error::{NovaError, NovaResult};
use raylib::prelude::*;
use vm::{
    memory_manager::{Object, ObjectType, VmData},
    state::{self, Draw},
};

fn pop_or_err(state: &mut state::State) -> NovaResult<VmData> {
    state.memory.stack.pop().ok_or(Box::new(NovaError::Runtime {
        msg: "Stack is empty".into(),
    }))
}

fn pop_int(state: &mut state::State) -> NovaResult<i64> {
    match pop_or_err(state)? {
        VmData::Int(v) => Ok(v),
        other => Err(Box::new(NovaError::Runtime {
            msg: format!("Expected Int, got {}", other).into(),
        })),
    }
}

#[allow(dead_code)]
fn pop_float(state: &mut state::State) -> NovaResult<f64> {
    match pop_or_err(state)? {
        VmData::Float(v) => Ok(v),
        other => Err(Box::new(NovaError::Runtime {
            msg: format!("Expected Float, got {}", other).into(),
        })),
    }
}

fn pop_string(state: &mut state::State) -> NovaResult<String> {
    match pop_or_err(state)? {
        VmData::Object(index) => {
            let s = state
                .memory
                .ref_from_heap(index)
                .and_then(|obj| obj.as_string().map(|s| s.to_string()))
                .ok_or(Box::new(NovaError::Runtime {
                    msg: "Expected a string object".into(),
                }))?;
            state.memory.dec(index);
            Ok(s)
        }
        _ => Err(Box::new(NovaError::Runtime {
            msg: "Expected a string on the stack".into(),
        })),
    }
}

#[allow(dead_code)]
fn pop_bool(state: &mut state::State) -> NovaResult<bool> {
    match pop_or_err(state)? {
        VmData::Bool(v) => Ok(v),
        other => Err(Box::new(NovaError::Runtime {
            msg: format!("Expected Bool, got {}", other).into(),
        })),
    }
}

/// Extract a Color from a Tuple/List object (Int, Int, Int) on the stack
fn pop_color(state: &mut state::State) -> NovaResult<Color> {
    match pop_or_err(state)? {
        VmData::Object(index) => {
            let (r, g, b) = {
                let obj =
                    state
                        .memory
                        .ref_from_heap(index)
                        .ok_or(Box::new(NovaError::Runtime {
                            msg: "Invalid heap reference for color".into(),
                        }))?;
                match &obj.object_type {
                    ObjectType::Tuple | ObjectType::List => {
                        if obj.data.len() < 3 {
                            return Err(Box::new(NovaError::Runtime {
                                msg: "Color tuple needs 3 elements".into(),
                            }));
                        }
                        let r = match obj.data[0] {
                            VmData::Int(v) => v as u8,
                            _ => {
                                return Err(Box::new(NovaError::Runtime {
                                    msg: "Color component must be Int".into(),
                                }))
                            }
                        };
                        let g = match obj.data[1] {
                            VmData::Int(v) => v as u8,
                            _ => {
                                return Err(Box::new(NovaError::Runtime {
                                    msg: "Color component must be Int".into(),
                                }))
                            }
                        };
                        let b = match obj.data[2] {
                            VmData::Int(v) => v as u8,
                            _ => {
                                return Err(Box::new(NovaError::Runtime {
                                    msg: "Color component must be Int".into(),
                                }))
                            }
                        };
                        (r, g, b)
                    }
                    _ => {
                        return Err(Box::new(NovaError::Runtime {
                            msg: "Expected a Tuple for color".into(),
                        }))
                    }
                }
            };
            state.memory.dec(index);
            Ok(Color::new(r, g, b, 255))
        }
        _ => Err(Box::new(NovaError::Runtime {
            msg: "Expected a color tuple object".into(),
        })),
    }
}

pub fn raylib_init(state: &mut state::State) -> NovaResult<()> {
    let fps = pop_int(state)? as u32;
    let height = pop_int(state)? as i32;
    let width = pop_int(state)? as i32;
    let title = pop_string(state)?;

    let (mut rl, thread) = raylib::init().size(width, height).title(&title).build();

    rl.set_target_fps(fps);

    state.raylib = Some(Rc::new(RefCell::new(rl)));
    state.raylib_thread = Some(thread);

    Ok(())
}

pub fn raylib_rendering(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let thread = state
        .raylib_thread
        .as_ref()
        .ok_or(Box::new(NovaError::Runtime {
            msg: "Raylib thread not initialized".into(),
        }))?;

    let should_close = {
        let rl = rl_rc.borrow();
        rl.window_should_close()
    };

    if should_close {
        state.memory.stack.push(VmData::Bool(false));
        return Ok(());
    }

    // Draw everything in the draw queue
    {
        let mut rl = rl_rc.borrow_mut();
        let mut d = rl.begin_drawing(thread);

        for draw_cmd in state.draw_queue.drain(..) {
            match draw_cmd {
                Draw::ClearBackground(color) => {
                    d.clear_background(color);
                }
                Draw::Text(text, x, y, size, color) => {
                    d.draw_text(&text, x, y, size, color);
                }
                Draw::FPS(x, y) => {
                    d.draw_fps(x, y);
                }
                Draw::Rectangle(x, y, w, h, color) => {
                    d.draw_rectangle(x, y, w, h, color);
                }
                Draw::Circle(x, y, radius, color) => {
                    d.draw_circle(x, y, radius, color);
                }
                Draw::Line(x1, y1, x2, y2, color) => {
                    d.draw_line(x1, y1, x2, y2, color);
                }
                Draw::LineThick(x1, y1, x2, y2, thick, color) => {
                    d.draw_line_ex(
                        Vector2::new(x1 as f32, y1 as f32),
                        Vector2::new(x2 as f32, y2 as f32),
                        thick,
                        color,
                    );
                }
                Draw::RectangleLines(x, y, w, h, color) => {
                    d.draw_rectangle_lines(x, y, w, h, color);
                }
                Draw::RoundedRectangle(x, y, w, h, roundness, color) => {
                    d.draw_rectangle_rounded(
                        Rectangle::new(x as f32, y as f32, w as f32, h as f32),
                        roundness,
                        8,
                        color,
                    );
                }
                Draw::CircleLines(x, y, radius, color) => {
                    d.draw_circle_lines(x, y, radius, color);
                }
                Draw::Triangle(x1, y1, x2, y2, x3, y3, color) => {
                    d.draw_triangle(
                        Vector2::new(x1, y1),
                        Vector2::new(x2, y2),
                        Vector2::new(x3, y3),
                        color,
                    );
                }
                Draw::Sprite(sprite_index, x, y) => {
                    if let Some(texture) = state.sprites.get(sprite_index) {
                        d.draw_texture(texture.as_ref(), x, y, Color::WHITE);
                    }
                }
            }
        }
    }

    state.memory.stack.push(VmData::Bool(true));
    Ok(())
}

pub fn raylib_draw_text(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let size = pop_int(state)? as i32;
    let y = pop_int(state)? as i32;
    let x = pop_int(state)? as i32;
    let text = pop_string(state)?;

    state.draw_queue.push(Draw::Text(text, x, y, size, color));
    Ok(())
}

pub fn raylib_clear(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    state.draw_queue.push(Draw::ClearBackground(color));
    Ok(())
}

pub fn raylib_draw_fps(state: &mut state::State) -> NovaResult<()> {
    let y = pop_int(state)? as i32;
    let x = pop_int(state)? as i32;
    state.draw_queue.push(Draw::FPS(x, y));
    Ok(())
}

pub fn raylib_get_mouse_position(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let (x, y) = {
        let rl = rl_rc.borrow();
        let pos = rl.get_mouse_position();
        (pos.x as i64, pos.y as i64)
    };

    // Create a tuple (Int, Int)
    let tuple = Object {
        object_type: ObjectType::Tuple,
        table: Default::default(),
        data: vec![VmData::Int(x), VmData::Int(y)],
    };
    let idx = state.memory.allocate(tuple);
    state.memory.stack.push(VmData::Object(idx));
    Ok(())
}

pub fn raylib_draw_rectangle(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let h = pop_int(state)? as i32;
    let w = pop_int(state)? as i32;
    let y = pop_int(state)? as i32;
    let x = pop_int(state)? as i32;

    state.draw_queue.push(Draw::Rectangle(x, y, w, h, color));
    Ok(())
}

pub fn raylib_get_key_as_string(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let key = {
        let mut rl = rl_rc.borrow_mut();
        rl.get_key_pressed()
    };

    if let Some(key) = key {
        let key_string = format!("{:?}", key);
        // push_string allocates the string and pushes VmData::Object onto the stack
        state.memory.push_string(key_string);
    } else {
        // Push None
        state.memory.stack.push(VmData::None);
    }

    Ok(())
}

pub fn raylib_is_key_down(state: &mut state::State) -> NovaResult<()> {
    let key_name = pop_string(state)?;

    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let pressed = {
        let rl = rl_rc.borrow();
        if let Some(key) = string_to_key(&key_name) {
            rl.is_key_down(key)
        } else {
            false
        }
    };

    state.memory.stack.push(VmData::Bool(pressed));
    Ok(())
}

pub fn raylib_is_key_pressed(state: &mut state::State) -> NovaResult<()> {
    let key_name = pop_string(state)?;

    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let pressed = {
        let rl = rl_rc.borrow();
        if let Some(key) = string_to_key(&key_name) {
            rl.is_key_pressed(key)
        } else {
            false
        }
    };

    state.memory.stack.push(VmData::Bool(pressed));
    Ok(())
}

pub fn raylib_is_key_released(state: &mut state::State) -> NovaResult<()> {
    let key_name = pop_string(state)?;

    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let released = {
        let rl = rl_rc.borrow();
        if let Some(key) = string_to_key(&key_name) {
            rl.is_key_released(key)
        } else {
            false
        }
    };

    state.memory.stack.push(VmData::Bool(released));
    Ok(())
}

pub fn raylib_get_frame_time(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let dt = {
        let rl = rl_rc.borrow();
        rl.get_frame_time() as f64
    };

    state.memory.stack.push(VmData::Float(dt));
    Ok(())
}

pub fn raylib_load_texture(state: &mut state::State) -> NovaResult<()> {
    let frame_count = pop_int(state)?;
    let height = pop_int(state)?;
    let path = pop_string(state)?;

    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let thread = state
        .raylib_thread
        .as_ref()
        .ok_or(Box::new(NovaError::Runtime {
            msg: "Raylib thread not initialized".into(),
        }))?;

    // Resolve path relative to current_dir
    let full_path = state.current_dir.join(&path);
    let full_path_str = full_path.to_string_lossy().to_string();

    let texture = {
        let mut rl = rl_rc.borrow_mut();
        rl.load_texture(thread, &full_path_str).map_err(|e| {
            Box::new(NovaError::Runtime {
                msg: format!("Failed to load texture: {}", e).into(),
            })
        })?
    };

    let sprite_index = state.sprites.len();
    state.sprites.push(Rc::new(texture));

    // Create a struct-like object to represent Sprite
    // With fields: index, width, height, frame_count
    let sprite_obj = Object {
        object_type: ObjectType::Struct("Sprite".to_string()),
        table: {
            let mut t = std::collections::HashMap::new();
            t.insert("index".to_string(), 0);
            t.insert("width".to_string(), 1);
            t.insert("height".to_string(), 2);
            t.insert("frame_count".to_string(), 3);
            t
        },
        data: vec![
            VmData::Int(sprite_index as i64),
            VmData::Int(state.sprites[sprite_index].width() as i64),
            VmData::Int(height),
            VmData::Int(frame_count),
        ],
    };
    state.memory.allocate(sprite_obj);
    Ok(())
}

pub fn raylib_draw_texture(state: &mut state::State) -> NovaResult<()> {
    let y = pop_int(state)? as i32;
    let x = pop_int(state)? as i32;

    // Pop the sprite object
    match pop_or_err(state)? {
        VmData::Object(index) => {
            let sprite_index = {
                let obj =
                    state
                        .memory
                        .ref_from_heap(index)
                        .ok_or(Box::new(NovaError::Runtime {
                            msg: "Invalid sprite object".into(),
                        }))?;
                match obj.data.first() {
                    Some(VmData::Int(idx)) => *idx as usize,
                    _ => {
                        return Err(Box::new(NovaError::Runtime {
                            msg: "Sprite object missing index".into(),
                        }))
                    }
                }
            };
            state.memory.dec(index);
            state.draw_queue.push(Draw::Sprite(sprite_index, x, y));
            Ok(())
        }
        _ => Err(Box::new(NovaError::Runtime {
            msg: "Expected a Sprite object".into(),
        })),
    }
}

pub fn raylib_draw_circle(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let radius = pop_int(state)? as i32;
    let y = pop_int(state)? as i32;
    let x = pop_int(state)? as i32;

    state
        .draw_queue
        .push(Draw::Circle(x, y, radius as f32, color));
    Ok(())
}

pub fn raylib_draw_line(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let y2 = pop_int(state)? as i32;
    let x2 = pop_int(state)? as i32;
    let y1 = pop_int(state)? as i32;
    let x1 = pop_int(state)? as i32;

    state.draw_queue.push(Draw::Line(x1, y1, x2, y2, color));
    Ok(())
}

pub fn raylib_get_time(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let time = {
        let rl = rl_rc.borrow();
        rl.get_time()
    };

    state.memory.stack.push(VmData::Float(time));
    Ok(())
}

pub fn raylib_is_mouse_button_down(state: &mut state::State) -> NovaResult<()> {
    let button_name = pop_string(state)?;

    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let pressed = {
        let rl = rl_rc.borrow();
        if let Some(button) = string_to_mouse_button(&button_name) {
            rl.is_mouse_button_down(button)
        } else {
            false
        }
    };

    state.memory.stack.push(VmData::Bool(pressed));
    Ok(())
}

pub fn raylib_get_mouse_button_released(state: &mut state::State) -> NovaResult<()> {
    let button_name = pop_string(state)?;

    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;

    let released = {
        let rl = rl_rc.borrow();
        if let Some(button) = string_to_mouse_button(&button_name) {
            rl.is_mouse_button_released(button)
        } else {
            false
        }
    };

    state.memory.stack.push(VmData::Bool(released));
    Ok(())
}

pub fn raylib_sleep(state: &mut state::State) -> NovaResult<()> {
    let ms = pop_int(state)?;
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    Ok(())
}

pub fn create_sprite_from_array(state: &mut state::State) -> NovaResult<()> {
    // Pop: pixel_list, frame_count, height, width
    let pixel_list_data = pop_or_err(state)?;
    let frame_count = pop_int(state)?;
    let height = pop_int(state)? as u32;
    let width = pop_int(state)? as u32;

    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let thread = state
        .raylib_thread
        .as_ref()
        .ok_or(Box::new(NovaError::Runtime {
            msg: "Raylib thread not initialized".into(),
        }))?;

    // Extract pixel data from the list of tuples
    let pixels: Vec<u8> =
        match pixel_list_data {
            VmData::Object(list_index) => {
                let mut pixel_bytes = Vec::new();
                {
                    let obj = state.memory.ref_from_heap(list_index).ok_or(Box::new(
                        NovaError::Runtime {
                            msg: "Invalid pixel list".into(),
                        },
                    ))?;
                    for item in &obj.data {
                        if let VmData::Object(tuple_index) = item {
                            let tuple_obj = state.memory.ref_from_heap(*tuple_index).ok_or(
                                Box::new(NovaError::Runtime {
                                    msg: "Invalid pixel tuple".into(),
                                }),
                            )?;
                            if tuple_obj.data.len() >= 3 {
                                if let (VmData::Int(r), VmData::Int(g), VmData::Int(b)) =
                                    (&tuple_obj.data[0], &tuple_obj.data[1], &tuple_obj.data[2])
                                {
                                    pixel_bytes.push(*r as u8);
                                    pixel_bytes.push(*g as u8);
                                    pixel_bytes.push(*b as u8);
                                    pixel_bytes.push(255u8); // alpha
                                }
                            }
                        }
                    }
                }
                state.memory.dec(list_index);
                pixel_bytes
            }
            _ => {
                return Err(Box::new(NovaError::Runtime {
                    msg: "Expected a list of color tuples".into(),
                }))
            }
        };

    // Create Image from raw pixel data
    let image = unsafe {
        let data_ptr = pixels.as_ptr() as *mut std::ffi::c_void;
        let raw = raylib::ffi::Image {
            data: data_ptr,
            width: width as i32,
            height: height as i32,
            mipmaps: 1,
            format: raylib::ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
        };
        Image::from_raw(raw)
    };

    let texture = {
        let mut rl = rl_rc.borrow_mut();
        rl.load_texture_from_image(thread, &image).map_err(|e| {
            Box::new(NovaError::Runtime {
                msg: format!("Failed to create texture from image: {}", e).into(),
            })
        })?
    };

    let sprite_index = state.sprites.len();
    state.sprites.push(Rc::new(texture));

    let sprite_obj = Object {
        object_type: ObjectType::Struct("Sprite".to_string()),
        table: {
            let mut t = std::collections::HashMap::new();
            t.insert("index".to_string(), 0);
            t.insert("width".to_string(), 1);
            t.insert("height".to_string(), 2);
            t.insert("frame_count".to_string(), 3);
            t
        },
        data: vec![
            VmData::Int(sprite_index as i64),
            VmData::Int(width as i64),
            VmData::Int(height as i64),
            VmData::Int(frame_count),
        ],
    };
    state.memory.allocate(sprite_obj);
    Ok(())
}

// ---------------------------------------------------------------
// Additional raylib functions
// ---------------------------------------------------------------

pub fn raylib_draw_rectangle_lines(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let h = pop_int(state)? as i32;
    let w = pop_int(state)? as i32;
    let y = pop_int(state)? as i32;
    let x = pop_int(state)? as i32;
    state
        .draw_queue
        .push(Draw::RectangleLines(x, y, w, h, color));
    Ok(())
}

pub fn raylib_draw_circle_lines(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let radius = pop_int(state)? as i32;
    let y = pop_int(state)? as i32;
    let x = pop_int(state)? as i32;
    state
        .draw_queue
        .push(Draw::CircleLines(x, y, radius as f32, color));
    Ok(())
}

pub fn raylib_draw_triangle(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let y3 = pop_int(state)? as f32;
    let x3 = pop_int(state)? as f32;
    let y2 = pop_int(state)? as f32;
    let x2 = pop_int(state)? as f32;
    let y1 = pop_int(state)? as f32;
    let x1 = pop_int(state)? as f32;
    state
        .draw_queue
        .push(Draw::Triangle(x1, y1, x2, y2, x3, y3, color));
    Ok(())
}

pub fn raylib_draw_line_thick(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let thick = pop_float(state)? as f32;
    let y2 = pop_int(state)? as i32;
    let x2 = pop_int(state)? as i32;
    let y1 = pop_int(state)? as i32;
    let x1 = pop_int(state)? as i32;
    state
        .draw_queue
        .push(Draw::LineThick(x1, y1, x2, y2, thick, color));
    Ok(())
}

pub fn raylib_get_screen_width(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let w = { rl_rc.borrow().get_screen_width() as i64 };
    state.memory.stack.push(VmData::Int(w));
    Ok(())
}

pub fn raylib_get_screen_height(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let h = { rl_rc.borrow().get_screen_height() as i64 };
    state.memory.stack.push(VmData::Int(h));
    Ok(())
}

pub fn raylib_set_target_fps(state: &mut state::State) -> NovaResult<()> {
    let fps = pop_int(state)? as u32;
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    rl_rc.borrow_mut().set_target_fps(fps);
    Ok(())
}

pub fn raylib_get_fps(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let fps = { rl_rc.borrow().get_fps() as i64 };
    state.memory.stack.push(VmData::Int(fps));
    Ok(())
}

pub fn raylib_measure_text(state: &mut state::State) -> NovaResult<()> {
    let size = pop_int(state)? as i32;
    let text = pop_string(state)?;
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let width = { rl_rc.borrow().measure_text(&text, size) as i64 };
    state.memory.stack.push(VmData::Int(width));
    Ok(())
}

pub fn raylib_get_mouse_wheel(state: &mut state::State) -> NovaResult<()> {
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let wheel = { rl_rc.borrow().get_mouse_wheel_move() as f64 };
    state.memory.stack.push(VmData::Float(wheel));
    Ok(())
}

pub fn raylib_is_key_up(state: &mut state::State) -> NovaResult<()> {
    let key_name = pop_string(state)?;
    let rl_rc = state.raylib.clone().ok_or(Box::new(NovaError::Runtime {
        msg: "Raylib not initialized".into(),
    }))?;
    let up = {
        let rl = rl_rc.borrow();
        if let Some(key) = string_to_key(&key_name) {
            rl.is_key_up(key)
        } else {
            true
        }
    };
    state.memory.stack.push(VmData::Bool(up));
    Ok(())
}

pub fn raylib_draw_rounded_rectangle(state: &mut state::State) -> NovaResult<()> {
    let color = pop_color(state)?;
    let roundness = pop_float(state)? as f32;
    let h = pop_int(state)? as i32;
    let w = pop_int(state)? as i32;
    let y = pop_int(state)? as i32;
    let x = pop_int(state)? as i32;
    state
        .draw_queue
        .push(Draw::RoundedRectangle(x, y, w, h, roundness, color));
    Ok(())
}

fn string_to_key(s: &str) -> Option<KeyboardKey> {
    match s {
        "Space" | "KEY_SPACE" => Some(KeyboardKey::KEY_SPACE),
        "Enter" | "KEY_ENTER" => Some(KeyboardKey::KEY_ENTER),
        "Escape" | "KEY_ESCAPE" => Some(KeyboardKey::KEY_ESCAPE),
        "Right" | "KeyRight" | "KEY_RIGHT" => Some(KeyboardKey::KEY_RIGHT),
        "Left" | "KeyLeft" | "KEY_LEFT" => Some(KeyboardKey::KEY_LEFT),
        "Up" | "KeyUp" | "KEY_UP" => Some(KeyboardKey::KEY_UP),
        "Down" | "KeyDown" | "KEY_DOWN" => Some(KeyboardKey::KEY_DOWN),
        "A" | "KeyA" | "KEY_A" => Some(KeyboardKey::KEY_A),
        "B" | "KeyB" | "KEY_B" => Some(KeyboardKey::KEY_B),
        "C" | "KeyC" | "KEY_C" => Some(KeyboardKey::KEY_C),
        "D" | "KeyD" | "KEY_D" => Some(KeyboardKey::KEY_D),
        "E" | "KeyE" | "KEY_E" => Some(KeyboardKey::KEY_E),
        "F" | "KeyF" | "KEY_F" => Some(KeyboardKey::KEY_F),
        "G" | "KeyG" | "KEY_G" => Some(KeyboardKey::KEY_G),
        "H" | "KeyH" | "KEY_H" => Some(KeyboardKey::KEY_H),
        "I" | "KeyI" | "KEY_I" => Some(KeyboardKey::KEY_I),
        "J" | "KeyJ" | "KEY_J" => Some(KeyboardKey::KEY_J),
        "K" | "KeyK" | "KEY_K" => Some(KeyboardKey::KEY_K),
        "L" | "KeyL" | "KEY_L" => Some(KeyboardKey::KEY_L),
        "M" | "KeyM" | "KEY_M" => Some(KeyboardKey::KEY_M),
        "N" | "KeyN" | "KEY_N" => Some(KeyboardKey::KEY_N),
        "O" | "KeyO" | "KEY_O" => Some(KeyboardKey::KEY_O),
        "P" | "KeyP" | "KEY_P" => Some(KeyboardKey::KEY_P),
        "Q" | "KeyQ" | "KEY_Q" => Some(KeyboardKey::KEY_Q),
        "R" | "KeyR" | "KEY_R" => Some(KeyboardKey::KEY_R),
        "S" | "KeyS" | "KEY_S" => Some(KeyboardKey::KEY_S),
        "T" | "KeyT" | "KEY_T" => Some(KeyboardKey::KEY_T),
        "U" | "KeyU" | "KEY_U" => Some(KeyboardKey::KEY_U),
        "V" | "KeyV" | "KEY_V" => Some(KeyboardKey::KEY_V),
        "W" | "KeyW" | "KEY_W" => Some(KeyboardKey::KEY_W),
        "X" | "KeyX" | "KEY_X" => Some(KeyboardKey::KEY_X),
        "Y" | "KeyY" | "KEY_Y" => Some(KeyboardKey::KEY_Y),
        "Z" | "KeyZ" | "KEY_Z" => Some(KeyboardKey::KEY_Z),
        "Zero" | "KEY_ZERO" => Some(KeyboardKey::KEY_ZERO),
        "One" | "KEY_ONE" => Some(KeyboardKey::KEY_ONE),
        "Two" | "KEY_TWO" => Some(KeyboardKey::KEY_TWO),
        "Three" | "KEY_THREE" => Some(KeyboardKey::KEY_THREE),
        "Four" | "KEY_FOUR" => Some(KeyboardKey::KEY_FOUR),
        "Five" | "KEY_FIVE" => Some(KeyboardKey::KEY_FIVE),
        "Six" | "KEY_SIX" => Some(KeyboardKey::KEY_SIX),
        "Seven" | "KEY_SEVEN" => Some(KeyboardKey::KEY_SEVEN),
        "Eight" | "KEY_EIGHT" => Some(KeyboardKey::KEY_EIGHT),
        "Nine" | "KEY_NINE" => Some(KeyboardKey::KEY_NINE),
        "Tab" | "KEY_TAB" => Some(KeyboardKey::KEY_TAB),
        "Backspace" | "KEY_BACKSPACE" => Some(KeyboardKey::KEY_BACKSPACE),
        "LeftShift" | "KEY_LEFT_SHIFT" => Some(KeyboardKey::KEY_LEFT_SHIFT),
        "RightShift" | "KEY_RIGHT_SHIFT" => Some(KeyboardKey::KEY_RIGHT_SHIFT),
        "LeftControl" | "KEY_LEFT_CONTROL" => Some(KeyboardKey::KEY_LEFT_CONTROL),
        "RightControl" | "KEY_RIGHT_CONTROL" => Some(KeyboardKey::KEY_RIGHT_CONTROL),
        _ => None,
    }
}

fn string_to_mouse_button(s: &str) -> Option<MouseButton> {
    match s {
        "Left" | "MOUSE_LEFT_BUTTON" => Some(MouseButton::MOUSE_BUTTON_LEFT),
        "Right" | "MOUSE_RIGHT_BUTTON" => Some(MouseButton::MOUSE_BUTTON_RIGHT),
        "Middle" | "MOUSE_MIDDLE_BUTTON" => Some(MouseButton::MOUSE_BUTTON_MIDDLE),
        _ => None,
    }
}

// ---------------------------------------------------------------
// Audio functions
// ---------------------------------------------------------------

/// raylib::initAudio() -> Void
/// Initialize the audio device. Must be called before loading sounds/music.
pub fn raylib_init_audio(state: &mut state::State) -> NovaResult<()> {
    if state.audio_initialized {
        return Ok(()); // already initialized
    }
    unsafe {
        raylib::ffi::InitAudioDevice();
    }
    if unsafe { raylib::ffi::IsAudioDeviceReady() } {
        state.audio_initialized = true;
        Ok(())
    } else {
        Err(Box::new(NovaError::Runtime {
            msg: "Failed to initialize audio device".into(),
        }))
    }
}

/// raylib::closeAudio() -> Void
/// Close the audio device.
pub fn raylib_close_audio(state: &mut state::State) -> NovaResult<()> {
    if !state.audio_initialized {
        return Ok(());
    }
    // Unload all sounds and music first
    for s in state.sounds.drain(..) {
        unsafe { raylib::ffi::UnloadSound(s) };
    }
    for m in state.music.drain(..) {
        unsafe { raylib::ffi::UnloadMusicStream(m) };
    }
    unsafe {
        raylib::ffi::CloseAudioDevice();
    }
    state.audio_initialized = false;
    Ok(())
}

/// raylib::setMasterVolume(volume: Float) -> Void
pub fn raylib_set_master_volume(state: &mut state::State) -> NovaResult<()> {
    let vol = pop_float(state)? as f32;
    unsafe { raylib::ffi::SetMasterVolume(vol) };
    Ok(())
}

/// raylib::loadSound(path: String) -> Int  (returns a sound index)
pub fn raylib_load_sound(state: &mut state::State) -> NovaResult<()> {
    let path = pop_string(state)?;
    if !state.audio_initialized {
        return Err(Box::new(NovaError::Runtime {
            msg: "Audio device not initialized. Call raylib::initAudio() first.".into(),
        }));
    }
    let full_path = state.current_dir.join(&path);
    let c_path = std::ffi::CString::new(full_path.to_string_lossy().as_ref()).map_err(|_| {
        Box::new(NovaError::Runtime {
            msg: "Invalid file path for loadSound".into(),
        })
    })?;
    let sound = unsafe { raylib::ffi::LoadSound(c_path.as_ptr()) };
    if unsafe { !raylib::ffi::IsSoundValid(sound) } {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Failed to load sound: {}", path).into(),
        }));
    }
    let idx = state.sounds.len();
    state.sounds.push(sound);
    state.memory.stack.push(VmData::Int(idx as i64));
    Ok(())
}

/// raylib::playSound(id: Int) -> Void
pub fn raylib_play_sound(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.sounds.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid sound id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::PlaySound(state.sounds[id]) };
    Ok(())
}

/// raylib::stopSound(id: Int) -> Void
pub fn raylib_stop_sound(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.sounds.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid sound id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::StopSound(state.sounds[id]) };
    Ok(())
}

/// raylib::pauseSound(id: Int) -> Void
pub fn raylib_pause_sound(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.sounds.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid sound id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::PauseSound(state.sounds[id]) };
    Ok(())
}

/// raylib::resumeSound(id: Int) -> Void
pub fn raylib_resume_sound(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.sounds.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid sound id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::ResumeSound(state.sounds[id]) };
    Ok(())
}

/// raylib::isSoundPlaying(id: Int) -> Bool
pub fn raylib_is_sound_playing(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.sounds.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid sound id: {}", id).into(),
        }));
    }
    let playing = unsafe { raylib::ffi::IsSoundPlaying(state.sounds[id]) };
    state.memory.stack.push(VmData::Bool(playing));
    Ok(())
}

/// raylib::setSoundVolume(id: Int, volume: Float) -> Void
pub fn raylib_set_sound_volume(state: &mut state::State) -> NovaResult<()> {
    let vol = pop_float(state)? as f32;
    let id = pop_int(state)? as usize;
    if id >= state.sounds.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid sound id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::SetSoundVolume(state.sounds[id], vol) };
    Ok(())
}

/// raylib::setSoundPitch(id: Int, pitch: Float) -> Void
pub fn raylib_set_sound_pitch(state: &mut state::State) -> NovaResult<()> {
    let pitch = pop_float(state)? as f32;
    let id = pop_int(state)? as usize;
    if id >= state.sounds.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid sound id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::SetSoundPitch(state.sounds[id], pitch) };
    Ok(())
}

/// raylib::loadMusic(path: String) -> Int  (returns a music index)
pub fn raylib_load_music(state: &mut state::State) -> NovaResult<()> {
    let path = pop_string(state)?;
    if !state.audio_initialized {
        return Err(Box::new(NovaError::Runtime {
            msg: "Audio device not initialized. Call raylib::initAudio() first.".into(),
        }));
    }
    let full_path = state.current_dir.join(&path);
    let c_path = std::ffi::CString::new(full_path.to_string_lossy().as_ref()).map_err(|_| {
        Box::new(NovaError::Runtime {
            msg: "Invalid file path for loadMusic".into(),
        })
    })?;
    let music = unsafe { raylib::ffi::LoadMusicStream(c_path.as_ptr()) };
    if unsafe { !raylib::ffi::IsMusicValid(music) } {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Failed to load music: {}", path).into(),
        }));
    }
    let idx = state.music.len();
    state.music.push(music);
    state.memory.stack.push(VmData::Int(idx as i64));
    Ok(())
}

/// raylib::playMusic(id: Int) -> Void
pub fn raylib_play_music(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::PlayMusicStream(state.music[id]) };
    Ok(())
}

/// raylib::updateMusic(id: Int) -> Void
/// Must be called every frame for music to keep playing.
pub fn raylib_update_music(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::UpdateMusicStream(state.music[id]) };
    Ok(())
}

/// raylib::stopMusic(id: Int) -> Void
pub fn raylib_stop_music(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::StopMusicStream(state.music[id]) };
    Ok(())
}

/// raylib::pauseMusic(id: Int) -> Void
pub fn raylib_pause_music(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::PauseMusicStream(state.music[id]) };
    Ok(())
}

/// raylib::resumeMusic(id: Int) -> Void
pub fn raylib_resume_music(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::ResumeMusicStream(state.music[id]) };
    Ok(())
}

/// raylib::isMusicPlaying(id: Int) -> Bool
pub fn raylib_is_music_playing(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    let playing = unsafe { raylib::ffi::IsMusicStreamPlaying(state.music[id]) };
    state.memory.stack.push(VmData::Bool(playing));
    Ok(())
}

/// raylib::setMusicVolume(id: Int, volume: Float) -> Void
pub fn raylib_set_music_volume(state: &mut state::State) -> NovaResult<()> {
    let vol = pop_float(state)? as f32;
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::SetMusicVolume(state.music[id], vol) };
    Ok(())
}

/// raylib::setMusicPitch(id: Int, pitch: Float) -> Void
pub fn raylib_set_music_pitch(state: &mut state::State) -> NovaResult<()> {
    let pitch = pop_float(state)? as f32;
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::SetMusicPitch(state.music[id], pitch) };
    Ok(())
}

/// raylib::getMusicLength(id: Int) -> Float
pub fn raylib_get_music_length(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    let len = unsafe { raylib::ffi::GetMusicTimeLength(state.music[id]) };
    state.memory.stack.push(VmData::Float(len as f64));
    Ok(())
}

/// raylib::getMusicTimePlayed(id: Int) -> Float
pub fn raylib_get_music_time_played(state: &mut state::State) -> NovaResult<()> {
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    let time = unsafe { raylib::ffi::GetMusicTimePlayed(state.music[id]) };
    state.memory.stack.push(VmData::Float(time as f64));
    Ok(())
}

/// raylib::seekMusic(id: Int, position: Float) -> Void
pub fn raylib_seek_music(state: &mut state::State) -> NovaResult<()> {
    let pos = pop_float(state)? as f32;
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    unsafe { raylib::ffi::SeekMusicStream(state.music[id], pos) };
    Ok(())
}

/// raylib::setMusicLooping(id: Int, looping: Bool) -> Void
pub fn raylib_set_music_looping(state: &mut state::State) -> NovaResult<()> {
    let looping = match pop_or_err(state)? {
        VmData::Bool(b) => b,
        _ => {
            return Err(Box::new(NovaError::Runtime {
                msg: "Expected Bool for looping parameter".into(),
            }))
        }
    };
    let id = pop_int(state)? as usize;
    if id >= state.music.len() {
        return Err(Box::new(NovaError::Runtime {
            msg: format!("Invalid music id: {}", id).into(),
        }));
    }
    state.music[id].looping = looping;
    Ok(())
}
