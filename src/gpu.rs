use sdl2::{
    keyboard::TextInputUtil,
    pixels::{PixelFormat, PixelFormatEnum},
    render::{Canvas, Texture, TextureCreator},
    surface,
    video::{Window, WindowContext},
    EventPump, Sdl, VideoSubsystem,
};

use crate::ExecutionError;
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
    pub fb_size: u64, // should be result of fb_width*fb_height
    pub fb_width: u32,
    pub fb_height: u32,
}
impl GPU {
    pub fn new(fb_ptr: u64, fb_width: u32, fb_height: u32) -> Result<Self, ExecutionError> {
        let sdl_backend = sdl2::init()
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        let video = sdl_backend
            .video()
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        let event_pump = sdl_backend
            .event_pump()
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;

        let input = video.text_input();
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

        renderer
            .set_logical_size(300, 300)
            .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        // renderer
        //     .set_integer_scale(true)
        //     .map_err(|e| ExecutionError::new(format!("failed to initialize gpu backend: {e}")))?;
        let mut frame_buffer: *mut Texture = Box::leak(Box::new(
            unsafe {
                texture_creator.as_ref().unwrap().create_texture_streaming(
                    None,
                    fb_width as u32,
                    fb_height as u32,
                )
            }
            .map_err(|e| ExecutionError::new(format!("failed to initialize framebuffer: {e}")))?,
        ));
        println!("initialized gpu");
        Ok(Self {
            sdl_backend,
            video,
            renderer,
            texture_creator,
            frame_buffer,
            input,
            event_pump,
            fb_size: (fb_width as u64 * fb_height as u64),
            fb_width,
            fb_height,
            stdmem_frame_buffer_ptr: fb_ptr,
        })
    }
    pub fn draw(&mut self, fb: &[u8]) -> Result<(), ExecutionError> {
        // let factory = self.renderer.texture_creator();
        unsafe {
            if let Some(gpu_fb) = self.frame_buffer.as_mut() {
                gpu_fb
                    .update(None, fb, self.fb_width as usize)
                    .map_err(|e| {
                        ExecutionError::new(format!("failed to write to gpu framebuffer: {e}"))
                    })?;
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
    // tmp, might be integrated into real event poller
    pub fn handle_responsive(&mut self) -> Result<(), ExecutionError> {
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    return Err(ExecutionError::new(
                        "GPU Framebuffer terminated (host window closed by user): shutting down."
                            .to_string(),
                    ))
                }
                _ => continue,
            }
        }
        Ok(())
    }
}
