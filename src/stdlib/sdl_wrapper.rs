//! SDL2 wrapper for Wheel
//! Provides safe interface to SDL2 functionality

#[cfg(feature = "sdl")]
pub mod sdl {
    use std::ffi::CString;
    use sdl2::Sdl;
    use sdl2::video::Window;
    use sdl2::render::Canvas;
    use std::sync::Mutex;

    lazy_static::lazy_static! {
        static ref SDL_CONTEXT: Mutex<Option<Sdl>> = Mutex::new(None);
        static ref SDL_CANVAS: Mutex<Option<Canvas<Window>>> = Mutex::new(None);
    }

    pub fn init() -> i64 {
        match sdl2::init() {
            Ok(ctx) => {
                *SDL_CONTEXT.lock().unwrap() = Some(ctx);
                1
            }
            Err(_) => 0,
        }
    }

    pub fn create_window(width: i32, height: i32, title_ptr: i64) -> i64 {
        let title = unsafe {
            let ptr = title_ptr as *const u8;
            std::ffi::CStr::from_ptr(ptr as *const i8)
                .to_string_lossy()
                .into_owned()
        };

        if let Ok(ctx) = SDL_CONTEXT.lock() {
            if let Some(sdl) = ctx.as_ref() {
                if let Ok(video) = sdl.video() {
                    if let Ok(window) = video.window(&title, width as u32, height as u32).build() {
                        if let Ok(canvas) = window.into_canvas().build() {
                            *SDL_CANVAS.lock().unwrap() = Some(canvas);
                            return 1;
                        }
                    }
                }
            }
        }
        0
    }

    pub fn draw_pixel(x: i32, y: i32, r: u8, g: u8, b: u8) -> i64 {
        if let Ok(mut canvas) = SDL_CANVAS.lock() {
            if let Some(c) = canvas.as_mut() {
                c.set_draw_color(sdl2::pixels::Color::RGB(r, g, b));
                if c.draw_point((x, y)).is_ok() {
                    return 1;
                }
            }
        }
        0
    }

    pub fn draw_rect(x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) -> i64 {
        if let Ok(mut canvas) = SDL_CANVAS.lock() {
            if let Some(c) = canvas.as_mut() {
                c.set_draw_color(sdl2::pixels::Color::RGB(r, g, b));
                let rect = sdl2::rect::Rect::new(x, y, w as u32, h as u32);
                if c.fill_rect(rect).is_ok() {
                    return 1;
                }
            }
        }
        0
    }

    pub fn clear(r: u8, g: u8, b: u8) -> i64 {
        if let Ok(mut canvas) = SDL_CANVAS.lock() {
            if let Some(c) = canvas.as_mut() {
                c.set_draw_color(sdl2::pixels::Color::RGB(r, g, b));
                c.clear();
                return 1;
            }
        }
        0
    }

    pub fn present() -> i64 {
        if let Ok(mut canvas) = SDL_CANVAS.lock() {
            if let Some(c) = canvas.as_mut() {
                c.present();
                return 1;
            }
        }
        0
    }

    pub fn destroy_window() -> i64 {
        *SDL_CANVAS.lock().unwrap() = None;
        1
    }

    pub fn quit() -> i64 {
        *SDL_CANVAS.lock().unwrap() = None;
        *SDL_CONTEXT.lock().unwrap() = None;
        1
    }
}

#[cfg(not(feature = "sdl"))]
pub mod sdl {
    pub fn init() -> i64 { -1 }
    pub fn create_window(_w: i32, _h: i32, _t: i64) -> i64 { -1 }
    pub fn draw_pixel(_x: i32, _y: i32, _r: u8, _g: u8, _b: u8) -> i64 { -1 }
    pub fn draw_rect(_x: i32, _y: i32, _w: i32, _h: i32, _r: u8, _g: u8, _b: u8) -> i64 { -1 }
    pub fn clear(_r: u8, _g: u8, _b: u8) -> i64 { -1 }
    pub fn present() -> i64 { -1 }
    pub fn destroy_window() -> i64 { -1 }
    pub fn quit() -> i64 { -1 }
}
