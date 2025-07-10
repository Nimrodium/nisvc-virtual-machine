use std::{
    collections::HashMap,
    fs::{File, Metadata},
    io::{stderr, stdin, stdout, Read, Seek, Stderr, Stdin, Stdout, Write},
    time::Duration,
};

use crossterm::style::Stylize;
use sdl2::{libc::MS_SILENT, sys::exit};

use crate::{
    constant::{DEFAULT_CLOCK_SPEED, MEM_HEAP, MEM_INVALID, MEM_STACK, MEM_STATIC, STACK_POINTER},
    cpu::CPU,
    kernel_log, ExecutionError, _kernel_log,
    gpu::GPU,
};

pub static mut KERNEL_LOG: bool = false;
/// - `0x01..0x30`: nhk interrupts
/// - `0x31..0xfe`: program defined interrupts
/// - `0xff`: hard execution stop
pub struct Kernel {
    pub system: CPU,
    pub gpu: Option<GPU>,
    clock_speed: f32,
    user_interrupt_vector: [u64; 205],
    breakpoint_vector: Vec<u64>,
    file_descriptor_vector: HashMap<u64, IOInterface>,
    cmdline: Vec<String>,
    next_fd: u64,
    cores_dumped: usize,
    // frame_buffer_ptr: u64,
}
impl Kernel {
    pub fn new(cmdline: Vec<String>, heap: u64, stack: u64, clock_speed: f32) -> Self {
        let mut file_descriptor_vector = HashMap::new();
        file_descriptor_vector.insert(0, IOInterface::Stdin(stdin()));
        file_descriptor_vector.insert(1, IOInterface::Stdout(stdout()));
        file_descriptor_vector.insert(2, IOInterface::Stderr(stderr()));
        // let mut gpu_test = match GPU::new(0, 200, 200) {
        //     Ok(g) => g,
        //     Err(e) => {
        //         panic!("{e}")
        //     }
        // };
        Self {
            system: CPU::new(heap, stack),
            gpu: None,
            clock_speed,
            user_interrupt_vector: [0; 205],
            breakpoint_vector: Vec::new(),
            file_descriptor_vector,
            next_fd: 3,
            cores_dumped: 0,
            cmdline,
        }
    }

    fn resolve_user_interrupt(&self, code: u8) -> u64 {
        let real = code - 0x31;
        self.user_interrupt_vector[real as usize]
    }

    fn handle_interrupt(&mut self, code: u8) -> Result<(), ExecutionError> {
        match code {
            0x01 => {
                // file open

                let str_len = self.system.pop()?;
                let str_ptr = self.system.pop()?;
                let str_bytes = self.system.memory.read(str_ptr, str_len)?;
                let path = String::from_utf8_lossy(&str_bytes);
                kernel_log!("open(2) {path}");
                let file_descriptor = self.open_file(&path)?;
                self.system.push(file_descriptor)?;
                Ok(())
            }
            0x02 => {
                // file write
                kernel_log!("write(3)");
                let buf_len = self.system.pop()?;
                let buf_ptr = self.system.pop()?;
                let file_descriptor = self.system.pop()?;
                let buffer = self.system.memory.read(buf_ptr, buf_len)?;
                // kernel_log!("write buffer: {:?}, len: {}", buffer, buffer.len());

                self.write_file(file_descriptor, &buffer)?;
                Ok(())
            }
            0x03 => {
                //file read
                kernel_log!("read(3)");
                let n = self.system.pop()?;
                let ptr = self.system.pop()?;
                let fd = self.system.pop()?;
                let buf = self.read_file(fd, n)?;
                self.system.memory.write(ptr, &buf)?;
                Ok(())
            }
            0x04 => {
                //file seek
                kernel_log!("seek");
                todo!()
            }
            0x05 => {
                // file close

                let fd = self.system.pop()?;
                kernel_log!("close({fd})");
                self.close_file(fd)?;
                Ok(())
            }
            0x06 => {
                // silence print
                unsafe { KERNEL_LOG = false };
                Ok(())
            }
            0x07 => {
                // rawtty toggle
                kernel_log!("rawtty switch");
                todo!()
            }
            0x08 => {
                // tty cursor relative x y
                kernel_log!("tty relative cursor");
                todo!()
            }
            0x09 => {
                // tty cursor absolute x y
                kernel_log!("tty absolute cursor");
                todo!()
            }
            0x0a => {
                // malloc
                kernel_log!("malloc(1)");
                let size = self.system.pop()?;
                let ptr = self.system.memory.malloc(size)?;
                self.system.push(ptr)
            }
            0x0b => {
                // realloc
                kernel_log!("realloc(2)");
                let new_size = self.system.pop()?;
                let ptr = self.system.pop()?;
                let new_ptr = self.system.memory.realloc(ptr, new_size)?;
                self.system.push(new_ptr)
            }
            0x0c => {
                // free
                kernel_log!("free(1)");
                let ptr = self.system.pop()?;
                self.system.memory.free(ptr)
            }
            0x0d => {
                // memcpy
                kernel_log!("memcpy(3)");
                let n = self.system.pop()?;
                let src = self.system.pop()?;
                let dest = self.system.pop()?;
                self.system.memory.memcpy(dest, src, n)
            }
            0x0e => {
                // memset
                kernel_log!("memset(3)");
                let n = self.system.pop()?;
                let src = self.system.pop()?;
                let dest = self.system.pop()?;
                self.system.memory.memset(dest, src as u8, n)
            }
            0x0f => {
                kernel_log!("init_fb(3)");
                if let Some(mut gpu) = self.gpu.as_mut() {
                    gpu.free_fb();
                }
                let mode = self.system.pop()?;
                let height = self.system.pop()?;
                let width = self.system.pop()?;
                let frame_buffer_ptr = self.system.pop()?;
                self.gpu = Some(GPU::new(
                    frame_buffer_ptr,
                    width as u32,
                    height as u32,
                    mode as u8,
                )?);
                Ok(())
            }
            0x10 => {
                kernel_log!("draw_fb(0)");
                self.gpu_fb_refresh()
            }
            0x11 => {
                kernel_log!("get_fb_ptr(0)");
                if let Some(gpu) = self.gpu.as_ref() {
                    self.system.push(gpu.stdmem_frame_buffer_ptr)?;
                }
                Ok(())
            }
            0x12 => {
                kernel_log!("get_file_length(1)");
                let file_descriptor = self.system.pop()?;
                let metadata = self.stat_file(file_descriptor)?;
                self.system.push(metadata.len())
            }
            0x13 => {
                kernel_log!("dump(0)");
                self.core_dump()
            }
            _ => {
                return Err(ExecutionError::new(format!(
                    "unexpected interrupt {code:#x}"
                )))
            }
            0x15 => {
                kernel_log!("arg_argc");
                self.system.push(self.cmdline.len() as u64)?;
                Ok(())
            }
            0x16 => {
                let arg_idx = self.system.pop()?;
                kernel_log!("get_argv({arg_idx})");
                let arg = self
                    .cmdline
                    .get(arg_idx as usize)
                    .ok_or(ExecutionError::new(format!(
                        "argv[{arg_idx}] out of bounds; argc: {}",
                        self.cmdline.len()
                    )))?;
                let len = arg.len() as u64;
                let ptr = self.system.memory.malloc(len)?;
                self.system.memory.write(ptr, arg.as_bytes())?;
                self.system.push(ptr)?;
                self.system.push(len)?;
                Ok(())
            }
            0x17 => {
                let addr = self.system.pop()?;
                kernel_log!("memquery({addr})");
                let region = self.system.memory.memquery(addr);
                self.system.push(region as u64)?;
                Ok(())
            }
        }
    }

    pub fn run(&mut self) -> Result<(), ExecutionError> {
        self.core_dump()?;
        let millis = (1000.0 * (1.0 / self.clock_speed)) as u64;
        // println!("Cycle {millis}ms");
        let cycle_duration = Duration::from_millis(millis);
        println!("cycle duration: {millis}ms from {}Hz", self.clock_speed);
        loop {
            std::thread::sleep(cycle_duration);
            self.system.step()?;
            match self.system.pending_interrupt {
                0x00 => continue,
                0x14 => break,
                _ => {
                    // kernel_log!("decoding {:#x}", self.system.pending_interrupt);
                    self.handle_interrupt(self.system.pending_interrupt)?;
                    self.system.pending_interrupt = 0;
                }
            }
        }

        if let Some(gpu) = self.gpu.as_mut() {
            loop {
                if gpu.quit_loop() {
                    break;
                }
            }
        }
        Ok(())
    }
    pub fn gpu_fb_refresh(&mut self) -> Result<(), ExecutionError> {
        // let gpu = self.gpu.as_mut().unwrap();
        // let frame_buffer = self
        //     .system
        //     .memory
        //     .read(gpu.stdmem_frame_buffer_ptr, gpu.fb_size)?;
        if let Some(gpu) = self.gpu.as_mut() {
            let end_addr = gpu.stdmem_frame_buffer_ptr as usize + gpu.fb_size as usize;
            let frame_buffer =
                &self.system.memory.physical[gpu.stdmem_frame_buffer_ptr as usize..end_addr];
            gpu.draw(frame_buffer)?;
        } else {
            kernel_log!("refresh call ignored: gpu not initialized");
        }
        Ok(())
    }
    pub fn core_dump(&mut self) -> Result<(), ExecutionError> {
        const CORE: &str = "nisvc.core";
        let mut core_file = File::create(format!("{CORE}.{}", self.cores_dumped))
            .map_err(|e| ExecutionError::new(format!("failed to dump core: {e}")))?;
        core_file
            .write_all(&self.system.memory.physical)
            .map_err(|e| ExecutionError::new(format!("failed to dump core: {e}")))?;
        println!("{}", "core dumped".on_red());
        self.cores_dumped += 1;
        Ok(())
    }

    // file IO
    fn get_interface(&mut self, file_descriptor: u64) -> Result<&mut IOInterface, ExecutionError> {
        self.file_descriptor_vector
            .get_mut(&file_descriptor)
            .ok_or(ExecutionError::new(format!(
                "not a valid file descriptor: `{file_descriptor}`"
            )))
    }
    fn open_file(&mut self, path: &str) -> Result<u64, ExecutionError> {
        let file = File::open(path)
            .map_err(|e| ExecutionError::new(format!("could not open file `{path}`: {e}")))?;

        // file.read_to_end(&mut buf)
        //     .map_err(|e| ExecutionError::new(format!("could not read file `{path}`: {e}")))?;
        // let f = FileWrapper::Reader(Box::new(stdin()));
        self.file_descriptor_vector
            .insert(self.next_fd, IOInterface::File(file));
        self.next_fd += 1;
        Ok(self.next_fd - 1)
    }
    fn read_file(&mut self, file_descriptor: u64, n: u64) -> Result<Vec<u8>, ExecutionError> {
        self.get_interface(file_descriptor)?.read(n)
    }
    fn write_file(&mut self, file_descriptor: u64, buffer: &[u8]) -> Result<(), ExecutionError> {
        self.get_interface(file_descriptor)?.write(buffer)
    }
    fn stat_file(&mut self, file_descriptor: u64) -> Result<Metadata, ExecutionError> {
        self.get_interface(file_descriptor)?.stat()
    }
    fn close_file(&mut self, file_descriptor: u64) -> Result<(), ExecutionError> {
        if file_descriptor < 4 {
            return Err(ExecutionError::new(format!(
                "cannot close stdin/stdout/stderr"
            )));
        }
        self.file_descriptor_vector.remove(&file_descriptor);
        Ok(())
    }
}

enum IOInterface {
    Stdin(Stdin),
    Stdout(Stdout),
    Stderr(Stderr),
    File(File),
}
impl IOInterface {
    fn read(&mut self, n: u64) -> Result<Vec<u8>, ExecutionError> {
        let mut buffer = vec![0; n as usize];
        let bytes_read = match self {
            IOInterface::Stdin(stdin) => stdin.read(&mut buffer),
            IOInterface::Stdout(_) => {
                return Err(ExecutionError::new(format!("cannot read from stdout")))
            }
            IOInterface::Stderr(_) => {
                return Err(ExecutionError::new(format!("cannot read from stderr")))
            }
            IOInterface::File(file) => file.read(&mut buffer),
        }
        .map_err(|e| ExecutionError::new(format!("failed to read from io stream: `{e}`")))?;
        if bytes_read != n as usize {
            return Err(ExecutionError::new(format!("incomplete read from reader attempted to read {n} bytes but only read back {bytes_read}\n{buffer:#?}")));
        } else {
            Ok(buffer)
        }
        // Ok(buffer)
    }
    fn write(&mut self, buffer: &[u8]) -> Result<(), ExecutionError> {
        match self {
            IOInterface::Stdin(_) => {
                return Err(ExecutionError::new(format!("cannot write to stdin")))
            }
            IOInterface::Stdout(stdout) => stdout.write_all(buffer),
            IOInterface::Stderr(stderr) => stderr.write_all(buffer),
            IOInterface::File(file) => file.write_all(buffer),
        }
        .map_err(|e| ExecutionError::new(format!("failed to write to io stream: `{e}`")))?;
        Ok(())
    }
    fn seek(&mut self, offset: i64) -> Result<(), ExecutionError> {
        match self {
            IOInterface::Stdin(stdin) => {
                return Err(ExecutionError::new(format!("cannot seek stdin")))
            }
            IOInterface::Stdout(stdout) => {
                return Err(ExecutionError::new(format!("cannot seek stdout")))
            }
            IOInterface::Stderr(stderr) => {
                return Err(ExecutionError::new(format!("cannot seek stderr")))
            }
            IOInterface::File(file) => file.seek_relative(offset),
        }
        .map_err(|e| ExecutionError::new(format!("failed to seek io stream: `{e}`")))
    }
    fn stat(&mut self) -> Result<Metadata, ExecutionError> {
        match self {
            IOInterface::Stdin(stdin) => {
                return Err(ExecutionError::new(format!("cannot seek stdin")))
            }

            IOInterface::Stdout(stdout) => {
                return Err(ExecutionError::new(format!("cannot stat stdout")))
            }

            IOInterface::Stderr(stderr) => {
                return Err(ExecutionError::new(format!("cannot stat stderr")))
            }

            IOInterface::File(file) => file
                .metadata()
                .map_err(|e| ExecutionError::new(format!("failed to stat io stream: `{e}`"))),
        }
    }
}
