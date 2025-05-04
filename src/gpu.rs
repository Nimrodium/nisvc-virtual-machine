use std::{fs::File, io::Write};

use sdl2::{
    keyboard::TextInputUtil,
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureCreator},
    surface,
    video::{Window, WindowContext},
    EventPump, Sdl, VideoSubsystem,
};

use crate::{verbose_println, ExecutionError};
const DEFAULT_WINDOW_NAME: &str = "nisvc-system";
pub struct GPU {
    sdl_backend: Sdl,
    video: VideoSubsystem,
    pub renderer: Canvas<Window>,
    texture_creator: *mut TextureCreator<WindowContext>,
    frame_buffer: *mut Texture<'static>,
    input: TextInputUtil,
    event_pump: EventPump,
    pub stdmem_frame_buffer_ptr: u64,
    pub fb_size: u64, // should be result of (fb_width*fb_height*bpp)/8
    pub fb_width: u32,
    pub fb_height: u32,
}
impl GPU {
    pub fn new(
        fb_ptr: u64,
        fb_width: u32,
        fb_height: u32,
        mode: u8,
    ) -> Result<Self, ExecutionError> {
        let sdl_backend = sdl2::init()
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        let video = sdl_backend
            .video()
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        let event_pump = sdl_backend
            .event_pump()
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;

        let input = video.text_input();
        input.start();

        let window = video
            .window(DEFAULT_WINDOW_NAME, fb_width * 4, fb_height * 4)
            // .input_grabbed()
            .resizable()
            .build()
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        let mut renderer = window
            .into_canvas()
            .build()
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        let texture_creator: *mut TextureCreator<WindowContext> =
            Box::leak(Box::new(renderer.texture_creator()));

        // renderer
        //     .set_logical_size(300, 300)
        //     .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        // renderer
        //     .set_integer_scale(true)
        //     .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;

        // all modes except rgb24 are actually just greyscale atm
        // let (pixel_format, palette, pitch) = match mode {
        //     0 | 1 => (
        //         PixelFormatEnum::Index8,
        //         Some(build_palette_8bpp_greyscale()),
        //         fb_width,
        //     ), // text mode
        //     2 => (
        //         PixelFormatEnum::Index8,
        //         Some(build_palette_8bpp_greyscale()),
        //         fb_width,
        //     ), // 8bpp greyscale
        //     3 => (
        //         PixelFormatEnum::Index8,
        //         Some(build_palette_8bpp_greyscale()),
        //         fb_width,
        //     ), // 8bpp rgb NOT IMPLEMENTED
        //     4 => (PixelFormatEnum::RGB24, None, fb_width * 3),
        //     _ => return Err(ExecutionError::new(format!("unknown gpu mode {mode}"))),
        // };
        let mut frame_buffer: *mut Texture = Box::leak(Box::new(
            unsafe {
                texture_creator.as_ref().unwrap().create_texture_streaming(
                    PixelFormatEnum::RGB24,
                    fb_width as u32,
                    fb_height as u32,
                )
            }
            .map_err(|e| ExecutionError::new(format!("failed to initialize framebuffer: {e}")))?,
        ));
        // if let Some(palette) = palette {
        //     unsafe {
        //         if let Some(fb) = frame_buffer.as_mut() {
        //             fb.
        //         } else {
        //             panic!("gpu_frame_buffer dereference was null")
        //         }
        //     }
        // }
        println!("initialized gpu");
        Ok(Self {
            sdl_backend,
            video,
            renderer,
            texture_creator,
            frame_buffer,
            input,
            event_pump,
            fb_size: (fb_width as u64 * fb_height as u64 * 24) / 8,
            fb_width,
            fb_height,

            stdmem_frame_buffer_ptr: fb_ptr,
        })
    }
    pub fn free_fb(&mut self) {
        drop(unsafe { Box::from_raw(self.frame_buffer) });
        drop(unsafe { Box::from_raw(self.texture_creator) });
    }
    pub fn draw(&mut self, fb: &[u8]) -> Result<(), ExecutionError> {
        // let factory = self.renderer.texture_creator();
        unsafe {
            if let Some(gpu_fb) = self.frame_buffer.as_mut() {
                gpu_fb
                    .with_lock(None, |mut frame_buffer, pitch| {
                        match frame_buffer.write_all(fb) {
                            Ok(()) => (),
                            Err(e) => {
                                panic!("error while writing to internal framebuffer (sdl2 texture) {e}")
                            }
                        }
                    })
                    .map_err(|e| {
                        ExecutionError::new(format!("failed to write to gpu framebuffer: {e}"))
                    })?;
                // dump framebuffer
                let mut dmp = File::create("fb_dump.data").unwrap();
                dmp.write_all(fb).unwrap();
                self.renderer.copy(gpu_fb, None, None).map_err(|e| {
                    ExecutionError::new(format!("failed to write to gpu framebuffer: {e}"))
                })?;
            } else {
                // could replace with .expect() but i might propogate this if i actually ever encounter this error
                panic!("gpu_frame_buffer dereference was null")
            }
        }
        self.renderer.present();
        Ok(())
    }
    // tmp, might be integrated into a real event poller
    pub fn quit_loop(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    println!(
                        "GPU Framebuffer terminated (host window closed by user): shutting down."
                    );
                    return true;
                }
                _ => continue,
            }
        }
        false
    }
}

// could be a static but i do not want to write that 255 times, maybe rustc inlined it.
fn build_palette_8bpp_greyscale() -> Vec<Color> {
    let mut palette = Vec::<Color>::with_capacity(size_of::<Color>() * u8::MAX as usize);
    for i in 0..=u8::MAX {
        palette.push(Color::RGB(i, i, i));
    }
    palette
}
