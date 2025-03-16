// mmio.rs
// for io operations iteracted with using mmio
use std::{
    collections::HashMap,
    io::{stdout, Write},
    marker::PhantomData,
    process::exit,
    rc::Rc,
    sync::Arc,
    time::Duration,
};

const TITLE: &str = "NISVC";
const COLUMNS: u32 = 40;
const ROWS: u32 = 30;
const SCREEN_WIDTH: u32 = 400;
const SCREEN_HEIGHT: u32 = 300;
const FONT_PATH: &str = "../assets/Glass_TTY_VT220.ttf";
const FONT_SIZE: u16 = 20;
use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use sdl2::{
    self,
    event::Event,
    keyboard::{Keycode, Mod},
    pixels::Color,
    rect::Rect,
    render::{Canvas, TextureQuery},
    rwops::RWops,
    surface::Surface,
    ttf::Font,
    video::Window,
    EventPump, Sdl, VideoSubsystem,
};

use crate::{
    constant::{self},
    cpu::{VMError, VMErrorCode},
    verbose_println, very_verbose_println, DisplayMode,
};
// const PATH: &str = "/home/kyle/CodeSync/rust/nimcode/sdltest/Glass_TTY_VT220.ttf";

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Display {
    sdl_context: Sdl,
    // ttf_context: Sdl2TtfContext,
    video_subsystem: VideoSubsystem,
    pub canvas: Canvas<Window>,
}
impl Display {
    fn new(title: &str, dimensions: (u32, u32)) -> Result<Self, VMError> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(title, dimensions.0, dimensions.1)
            .position_centered()
            .hidden()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        // let ttf_context = sdl2::ttf::init().unwrap();
        Ok(Display {
            sdl_context,
            // ttf_context,
            video_subsystem,
            canvas,
        })
    }
}

// struct FontManager<'a> {
//     surface_cache: HashMap<char, Surface<'a>>,
// }
// impl FontManager {}

struct TextModeDisplay {
    display: Display,
    // columns: u32,
    // rows: u32,
    cursor: Cursor,
    // font: Font<'static, 'static>,
    // font: Font<'static, 'static>,
    // ttf_context: Arc<Sdl2TtfContext>,
    font_size: u16,
    font_path: String,
    cell_height: u32,
    cell_width: u32,
    ascii_cache: HashMap<char, Box<Surface<'static>>>,
    key_stack: Vec<u8>, // each read of the key MMIO address pops from this stack
    event_pump: EventPump,
    // font_file: &[u8; 0],
}
impl<'a> TextModeDisplay {
    fn generate_ascii_cache(font: &'a Font) -> Result<HashMap<char, Box<Surface<'a>>>, String> {
        very_verbose_println!("generating cache");

        let mut cache: HashMap<char, Box<Surface>> = HashMap::new();
        let lowest = 0;
        let highest = 255;

        for code in lowest..=highest {
            let ascii_code = code as u8 as char;
            // println!("{ascii_code}");
            // let mut ascii = String::new();
            // ascii.push(ascii_code);
            let ascii = if ascii_code.is_ascii_graphic() || ascii_code.is_ascii_whitespace() {
                ascii_code.to_string()
            } else {
                '?'.to_string()
            };
            let surface = font
                .render(&ascii)
                .blended(Color::RGB(255, 255, 255))
                .map_err(|e| e.to_string())?;
            cache.insert(ascii_code, Box::new(surface));
        }

        // todo!();
        Ok(cache)
    }
    fn new(
        title: &str,
        columns: u32,
        rows: u32,
        screen_width: u32,
        screen_height: u32,
        font_path: &str,
    ) -> Result<Self, VMError> {
        let cell_width = screen_width / columns;
        let cell_height = screen_height / rows + 6;

        let display = Display::new(title, (screen_width, screen_height))?;
        let event_pump = display.sdl_context.event_pump()?;

        let font_file_bytes = include_bytes!("../assets/Glass_TTY_VT220.ttf");
        // let ttf_context = Box::leak(Box::new(sdl2::ttf::init().map_err(|e| e.to_string())?));
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let ttf_box = Box::leak(Box::new(ttf_context));
        // let ttf_arc = Rc::new(ttf_context);
        let font = ttf_box.load_font_from_rwops(RWops::from_bytes(font_file_bytes)?, FONT_SIZE)?;
        let font_box = Box::leak(Box::new(font));
        // let cache = Box::leak(Box::new(TextModeDisplay::generate_ascii_cache(&font)?));
        let cache = TextModeDisplay::generate_ascii_cache(font_box)?;
        Ok(TextModeDisplay {
            display,
            // columns,
            // rows,
            cursor: Cursor::new(columns, rows),
            cell_height,
            cell_width,
            font_path: font_path.to_string(),
            font_size: 20,
            key_stack: vec![],
            // font,
            event_pump,
            ascii_cache: cache, // ttf_context: Arc::clone(&ttf_context),
        })
    }

    fn map_grid_coord(&self, cursor: (u32, u32)) -> (u32, u32) {
        let x = cursor.0 * self.cell_width;
        let y = cursor.1 * self.cell_height;
        (x, y)
    }
    /// writes a character to a grid position
    pub fn write(&mut self, ascii_code: u8, cursor: (u32, u32)) -> Result<(), String> {
        let abs_coord = self.map_grid_coord(cursor);
        // println!("write :: '{}' to {abs_coord:?}", ascii_code as char);
        // let surface = self.ascii_to_surface(ascii_code)?;

        let surface = match self.ascii_cache.get(&(ascii_code as char)) {
            Some(s) => &**s,
            None => {
                return Err(format!(
                    "could not get character for ascii {} : {}",
                    ascii_code as char, ascii_code
                ))
            }
        };
        let texture_creator = self.display.canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(surface)
            .map_err(|e| e.to_string())?;
        let TextureQuery { width, height, .. } = texture.query();
        let target = Rect::new(abs_coord.0 as i32, abs_coord.1 as i32, width, height);
        self.display.canvas.copy(&texture, None, Some(target))?;
        Ok(())
    }
    // fn ascii_to_surface(&self, ascii_code: u8) -> Result<Surface, String> {
    //     // let font = self.load_font()?;

    //     // let font = self.font.as_ref();
    //     let surface = self
    //         .font
    //         .render(&(ascii_code as char).to_string())
    //         .blended(Color::RGB(255, 255, 255))
    //         .map_err(|e| e.to_string())?;
    //     // todo!();
    //     Ok(surface)
    // }
    /// listens for key presses and returns a translated real value.
    pub fn key_processor(&mut self) {
        for event in self.event_pump.poll_iter() {
            let (keycode, keymod) = match event {
                Event::KeyDown {
                    timestamp,
                    window_id,
                    keycode,
                    scancode,
                    keymod,
                    repeat,
                } => (keycode.unwrap(), keymod),
                Event::Quit { .. } => exit(0),
                _ => continue,
            };
            let key = match keycode {
                Keycode::Backspace => {
                    println!("backspace");
                    '\x08'
                }
                Keycode::Return => {
                    println!("return");
                    '\n'
                }
                Keycode::LShift => continue,
                Keycode::RShift => continue,
                Keycode::Tab => '\t',
                Keycode::Space => ' ',
                Keycode::Left => {
                    println!("left");
                    128 as char
                }
                Keycode::Right => {
                    println!("right");
                    129 as char
                }
                Keycode::Up => {
                    println!("up");
                    130 as char
                }
                Keycode::Down => {
                    println!("down");
                    131 as char
                }
                _ => keycode
                    .to_string()
                    .chars()
                    .next()
                    .unwrap()
                    .to_ascii_lowercase(),
                // Keycode::Space => ' ',
            };
            // handle mod
            let modded_key = match keymod {
                Mod::LSHIFTMOD => {
                    println!("{keycode} upper to {}", key.to_ascii_uppercase());
                    match key {
                        '/' => '?',
                        ',' => '<',
                        '.' => '>',
                        ';' => ':',
                        '\'' => '"',
                        '[' => '{',
                        ']' => '}',
                        '-' => '_',
                        '=' => '+',
                        '\\' => '|',
                        '`' => '~',

                        _ => {
                            if !key.is_ascii_digit() {
                                key.to_ascii_uppercase()
                            } else {
                                match key {
                                    '1' => '!',
                                    '2' => '@',
                                    '3' => '#',
                                    '4' => '$',
                                    '5' => '%',
                                    '6' => '^',
                                    '7' => '&',
                                    '8' => '*',
                                    '9' => '(',
                                    '0' => ')',
                                    _ => continue,
                                }
                            }
                        }
                    }
                }
                Mod::NOMOD => key,
                _ => {
                    println!("idk man. {keymod:?}");
                    key
                }
            };
            println!("detected {modded_key} :: {}", modded_key as u8);
            self.key_stack.push(modded_key as u8)
        }
    }
}

struct Cursor {
    pub x: u32,
    pub y: u32,
    pub x_bound: u32,
    pub y_bound: u32,
}
impl Cursor {
    fn to_tuple(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    fn new(x_bound: u32, y_bound: u32) -> Self {
        Cursor {
            x: 0,
            y: 0,
            x_bound,
            y_bound,
        }
    }
    fn set(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
    fn left(&mut self) {
        self.x = (self.x.saturating_sub(1)) % self.x_bound;
        if self.x == 0 {
            self.y = self.y.saturating_sub(1);
        };
    }
    fn right(&mut self) {
        self.x = (self.x.saturating_add(1)) % self.x_bound;
        if self.x == 0 {
            self.y = self.y.saturating_add(1);
        };
    }
    fn up(&mut self) {
        self.y = self.y.saturating_add(1);
    }
    fn down(&mut self) {
        self.y = self.y.saturating_sub(1);
    }

    fn new_line(&mut self) {
        // (0, cursor.1 + 1)
        self.x = 0;
        self.y += 1;
    }
}

struct StdOutDisplay {
    stdout: std::io::Stdout,
    key_stack: Vec<u8>,
    cursor: Cursor,
}
impl StdOutDisplay {
    fn new() -> Result<Self, VMError> {
        let (x_bound, y_bound) = match terminal::size() {
            Ok(dimensions) => dimensions,
            Err(why) => {
                return Err(VMError::new(
                    VMErrorCode::DisplayInitializationError,
                    format!("failed to get terminal dimensions :: {why}"),
                ))
            }
        };
        Ok(Self {
            stdout: std::io::stdout(),
            key_stack: vec![],
            cursor: Cursor::new(x_bound as u32, y_bound as u32),
        })
    }

    fn enable_raw_mode() -> Result<(), VMError> {
        match terminal::enable_raw_mode() {
            Ok(()) => (),
            Err(why) => {
                return Err(VMError::new(
                    VMErrorCode::DisplayInitializationError,
                    format!("failed to enter raw terminal mode :: {why}"),
                ))
            }
        };
        match execute!(stdout(), EnterAlternateScreen) {
            Ok(()) => (),
            Err(why) => {
                return Err(VMError::new(
                    VMErrorCode::DisplayInitializationError,
                    format!("failed to enter alternate screen :: {why}"),
                ))
            }
        };
        Ok(())
    }
    fn disable_raw_mode() -> Result<(), VMError> {
        match terminal::disable_raw_mode() {
            Ok(()) => (),
            Err(why) => {
                return Err(VMError::new(
                    VMErrorCode::DisplayInitializationError,
                    format!("failed to leave raw terminal mode :: {why}"),
                ))
            }
        };
        match execute!(stdout(), LeaveAlternateScreen) {
            Ok(()) => (),
            Err(why) => {
                return Err(VMError::new(
                    VMErrorCode::GenericError,
                    format!("failed to leave raw terminal mode :: {why}"),
                ))
            }
        };
        Ok(())
    }
}

enum DisplayContainer {
    Window(TextModeDisplay),
    StdOut(StdOutDisplay),
}
trait DisplayManager {
    fn read_event_pump(&mut self) -> u8;

    fn show_display(&mut self) -> Result<(), VMError>;
    fn hide_display(&mut self) -> Result<(), VMError>;
    fn refresh_display(&mut self) -> Result<(), VMError>;

    fn write_at_cursor(&mut self, char_ascii: u8) -> Result<(), VMError>;
    fn move_cursor(&mut self, direction: Direction) -> Result<(), VMError>;
    fn set_cursor_x(&mut self, x: u32);
    fn set_cursor_y(&mut self, y: u32);
    fn cursor_new_line(&mut self);
    fn get_cursor(&mut self) -> (u32, u32);
}
impl DisplayManager for DisplayContainer {
    fn read_event_pump(&mut self) -> u8 {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.read_event_pump(),
            DisplayContainer::StdOut(std_out_display) => std_out_display.read_event_pump(),
        }
    }
    fn show_display(&mut self) -> Result<(), VMError> {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.show_display(),
            DisplayContainer::StdOut(std_out_display) => std_out_display.show_display(),
        }
    }

    fn hide_display(&mut self) -> Result<(), VMError> {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.hide_display(),
            DisplayContainer::StdOut(std_out_display) => std_out_display.hide_display(),
        }
    }

    fn refresh_display(&mut self) -> Result<(), VMError> {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.refresh_display(),
            DisplayContainer::StdOut(std_out_display) => std_out_display.refresh_display(),
        }
    }

    fn write_at_cursor(&mut self, char_ascii: u8) -> Result<(), VMError> {
        match self {
            DisplayContainer::Window(text_mode_display) => {
                text_mode_display.write_at_cursor(char_ascii)
            }
            DisplayContainer::StdOut(std_out_display) => {
                std_out_display.write_at_cursor(char_ascii)
            }
        }
    }

    fn move_cursor(&mut self, direction: Direction) -> Result<(), VMError> {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.move_cursor(direction),
            DisplayContainer::StdOut(std_out_display) => std_out_display.move_cursor(direction),
        }
    }

    fn set_cursor_x(&mut self, x: u32) {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.set_cursor_x(x),
            DisplayContainer::StdOut(std_out_display) => std_out_display.set_cursor_x(x),
        }
    }

    fn set_cursor_y(&mut self, y: u32) {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.set_cursor_y(y),
            DisplayContainer::StdOut(std_out_display) => std_out_display.set_cursor_y(y),
        }
    }

    fn cursor_new_line(&mut self) {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.cursor_new_line(),
            DisplayContainer::StdOut(std_out_display) => std_out_display.cursor_new_line(),
        }
    }

    fn get_cursor(&mut self) -> (u32, u32) {
        match self {
            DisplayContainer::Window(text_mode_display) => text_mode_display.get_cursor(),
            DisplayContainer::StdOut(std_out_display) => std_out_display.get_cursor(),
        }
    }
}
impl DisplayManager for TextModeDisplay {
    fn read_event_pump(&mut self) -> u8 {
        self.key_processor();
        let key = self.key_stack.pop();
        if let Some(k) = key {
            k
        } else {
            0
        }
    }

    fn show_display(&mut self) -> Result<(), VMError> {
        self.display.canvas.window_mut().show();
        Ok(())
    }

    fn hide_display(&mut self) -> Result<(), VMError> {
        self.display.canvas.window_mut().hide();
        Ok(())
    }

    fn refresh_display(&mut self) -> Result<(), VMError> {
        self.display.canvas.present();
        self.display.canvas.clear();
        Ok(())
    }

    fn write_at_cursor(&mut self, char_ascii: u8) -> Result<(), VMError> {
        match self.write(char_ascii, self.cursor.to_tuple()) {
            Ok(_) => Ok(()),
            Err(why) => Err(VMError::new(VMErrorCode::GenericError, why)),
        }
    }

    fn move_cursor(&mut self, direction: Direction) -> Result<(), VMError> {
        match direction {
            Direction::Up => self.cursor.up(),
            Direction::Down => self.cursor.down(),
            Direction::Left => self.cursor.left(),
            Direction::Right => self.cursor.right(),
        }
        Ok(())
    }

    fn set_cursor_x(&mut self, x: u32) {
        self.cursor.x = x
    }

    fn set_cursor_y(&mut self, y: u32) {
        self.cursor.y = y
    }

    fn cursor_new_line(&mut self) {
        self.cursor.new_line();
    }

    fn get_cursor(&mut self) -> (u32, u32) {
        self.cursor.to_tuple()
    }
}
impl DisplayManager for StdOutDisplay {
    fn read_event_pump(&mut self) -> u8 {
        let key = self.key_stack.pop();
        if let Some(k) = key {
            k
        } else {
            0
        }
    }
    fn show_display(&mut self) -> Result<(), VMError> {
        verbose_println!("mmio_show_display not supported in stdout mode");
        Ok(())
    }

    fn hide_display(&mut self) -> Result<(), VMError> {
        verbose_println!("mmio_hide_display not supported in stdout mode");
        Ok(())
    }

    fn refresh_display(&mut self) -> Result<(), VMError> {
        match std::io::stdout().flush() {
            Ok(()) => Ok(()),
            Err(why) => Err(VMError::new(
                VMErrorCode::GenericError,
                format!("mmio_refresh_display failed to flush stdout :: {why}"),
            )),
        }
    }

    fn write_at_cursor(&mut self, char_ascii: u8) -> Result<(), VMError> {
        todo!()
    }

    fn move_cursor(&mut self, direction: Direction) -> Result<(), VMError> {
        todo!()
    }

    fn set_cursor_x(&mut self, x: u32) {
        self.cursor.x = x;
    }

    fn set_cursor_y(&mut self, y: u32) {
        self.cursor.y = y;
    }

    fn cursor_new_line(&mut self) {
        self.cursor.new_line();
    }

    fn get_cursor(&mut self) -> (u32, u32) {
        self.cursor.to_tuple()
    }
}
pub struct MMIO {
    display: DisplayContainer,
    // cursor: Cursor,
    // // internal mmio chip registers
    // mr1: RegisterWidth,
    // mr2: RegisterWidth,
    // mr3: RegisterWidth,
    // mr4: RegisterWidth,
    // mr5: RegisterWidth,
}
impl MMIO {
    pub fn new(display_mode: DisplayMode) -> Result<Self, VMError> {
        verbose_println!("initializing IO");

        let display = match display_mode {
            DisplayMode::Window => {
                let inner = TextModeDisplay::new(
                    TITLE,
                    COLUMNS,
                    ROWS,
                    SCREEN_WIDTH,
                    SCREEN_HEIGHT,
                    FONT_PATH,
                )
                .map_err(|err| {
                    VMError::from(err).with_code(VMErrorCode::DisplayInitializationError)
                })?;
                DisplayContainer::Window(inner)
            }

            DisplayMode::Stdout => {
                let inner = StdOutDisplay::new()?;
                DisplayContainer::StdOut(inner)
            }
        };
        // let display =
        //     TextModeDisplay::new(TITLE, COLUMNS, ROWS, SCREEN_WIDTH, SCREEN_HEIGHT, FONT_PATH)
        //         .map_err(|err| {
        //             VMError::from(err).with_code(VMErrorCode::DisplayInitializationError)
        //         })?;
        // let cursor = Cursor::new(display.columns, display.rows);

        Ok(MMIO {
            display,
            // cursor,
            // mr1: INIT_VALUE,
            // mr2: INIT_VALUE,
            // mr3: INIT_VALUE,
            // mr4: INIT_VALUE,
            // mr5: INIT_VALUE,
        })
    }
    /// read addr 0x0 summons this function
    /// pops a value off the key_stack, if no values return 0
    fn read_key_mmio(&mut self) -> u8 {
        self.display.read_event_pump()
    }
    // fn write_string(&mut self){
    //     let string =
    // }

    pub fn mmio_read_handler(&mut self, address: constant::RegisterWidth) -> u8 {
        match address {
            0x0 => self.read_key_mmio(),
            _ => 0,
        }
    }
    pub fn mmio_write_handler(
        &mut self,
        address: constant::RegisterWidth,
        byte: u8,
    ) -> Result<(), VMError> {
        match address {
            // keyboard input address
            // 0x0 => self.display.key_stack.push(byte),
            // display state control address
            0x1 => match byte {
                0 => {
                    self.display.hide_display();
                    very_verbose_println!("mmio call :: hide display")
                }
                1 => {
                    self.display.show_display();
                    very_verbose_println!("mmio call :: show display")
                }
                2 => {
                    self.display.refresh_display();
                    very_verbose_println!("mmio call :: refresh display")
                }
                _ => (),
            },
            // cursor manual setting addresses
            0x2 => self.display.set_cursor_x(byte as u32),
            0x3 => self.display.set_cursor_y(byte as u32),
            // cursor control address
            0x4 => match byte {
                0 => self.display.move_cursor(Direction::Left)?,
                1 => self.display.move_cursor(Direction::Right)?,
                2 => self.display.move_cursor(Direction::Up)?,
                3 => self.display.move_cursor(Direction::Down)?,
                _ => (),
            },
            // display write at cursor
            0x5 => {
                if byte != 0 {
                    match byte {
                        10 => {
                            self.display.cursor_new_line();
                        }

                        _ => {
                            self.display.write_at_cursor(byte);
                            let (x, y) = self.display.get_cursor();
                            very_verbose_println!(
                                "mmio call :: write {byte} to display at ({},{})",
                                x,
                                y
                            );
                        }
                    }
                }
            }
            // // load string pointer
            // 0x6 => self.mr1 = byte as RegisterWidth,
            // // load string length
            // 0x7 => self.mr2 = byte as RegisterWidth,

            // // call write
            _ => (),
        };
        Ok(())
    }
}

// if i want to add functions for common operations then it cant be in the mmio handler entirely,
// it needs to be two scoped. mmio for io and mmfn or smth for those, like what the write syscall is
