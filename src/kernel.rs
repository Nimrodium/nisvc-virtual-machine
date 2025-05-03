use std::{
    collections::HashMap,
    fs::File,
    io::{stderr, stdin, stdout, Read, Seek, Stderr, Stdin, Stdout, Write},
};

use crossterm::style::Stylize;
use sdl2::{libc::MS_SILENT, sys::exit};

use crate::{constant::STACK_POINTER, cpu::CPU, kernel_log, ExecutionError, _kernel_log, gpu::GPU};

pub static mut SILENCE_KERNEL: bool = false;
/// - `0x01..0x30`: nhk interrupts
/// - `0x31..0xfe`: program defined interrupts
/// - `0xff`: hard execution stop
pub struct Kernel {
    pub system: CPU,
    pub gpu: Option<GPU>,
    user_interrupt_vector: [u64; 205],
    breakpoint_vector: Vec<u64>,
    file_descriptor_vector: HashMap<u64, IOInterface>,
    next_fd: u64,
    // frame_buffer_ptr: u64,
}
impl Kernel {
    pub fn new(heap: u64, stack: u64) -> Self {
        let mut file_descriptor_vector = HashMap::new();
        file_descriptor_vector.insert(0, IOInterface::Stdin(stdin()));
        file_descriptor_vector.insert(1, IOInterface::Stdout(stdout()));
        file_descriptor_vector.insert(2, IOInterface::Stderr(stderr()));
        let mut gpu_test = match GPU::new(0, 100, 100) {
            Ok(g) => g,
            Err(e) => {
                panic!("{e}")
            }
        };
        Self {
            system: CPU::new(heap, stack),
            gpu: Some(gpu_test),
            user_interrupt_vector: [0; 205],
            breakpoint_vector: Vec::new(),
            file_descriptor_vector,
            next_fd: 3,
            // frame_buffer_ptr:None,
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
                let str_ptr = self.system.pop()?;
                let str_len = self.system.pop()?;
                let str_bytes = self.system.memory.read(str_ptr, str_len)?;
                let path = String::from_utf8_lossy(&str_bytes);
                let file_descriptor = self.open_file(&path)?;
                self.system.push(file_descriptor)?;
                Ok(())
            }
            0x02 => {
                // file write
                kernel_log!("write");
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
                kernel_log!("read");
                todo!()
            }
            0x04 => {
                //file seek
                kernel_log!("seek");
                todo!()
            }
            0x05 => {
                // file close
                kernel_log!("close");
                todo!()
            }
            0x06 => {
                // silence print
                unsafe { SILENCE_KERNEL = true };
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
                kernel_log!("malloc");
                let size = self.system.pop()?;
                let ptr = self.system.memory.malloc(size)?;
                self.system.push(ptr)
            }
            0x0b => {
                // realloc
                kernel_log!("realloc");
                let new_size = self.system.pop()?;
                let ptr = self.system.pop()?;
                let new_ptr = self.system.memory.realloc(ptr, new_size)?;
                self.system.push(new_ptr)
            }
            0x0c => {
                // free
                kernel_log!("free");
                let ptr = self.system.pop()?;
                self.system.memory.free(ptr)
            }
            0x0d => {
                // memcpy
                kernel_log!("memcpy");
                let src = self.system.pop()?;
                let n = self.system.pop()?;
                let dest = self.system.pop()?;
                self.system.memory.memcpy(dest, src, n)
            }
            0x0e => {
                // memset
                kernel_log!("memset");
                let src = self.system.pop()?;
                let n = self.system.pop()?;
                let dest = self.system.pop()?;
                self.system.memory.memset(dest, src as u8, n)
            }

            _ => todo!(),
        }
    }

    pub fn run(&mut self) -> Result<(), ExecutionError> {
        loop {
            self.system.step()?;
            match self.system.pending_interrupt {
                0x00 => continue,
                0xff => break,
                _ => {
                    // kernel_log!("decoding {:#x}", self.system.pending_interrupt);
                    self.handle_interrupt(self.system.pending_interrupt)?;
                    self.system.pending_interrupt = 0;
                }
            }
        }
        self.gpu_fb_refresh()?;

        if let Some(gpu) = self.gpu.as_mut() {
            loop {
                gpu.handle_responsive()?;
            }
        }
        Ok(())
    }
    pub fn gpu_fb_refresh(&mut self) -> Result<(), ExecutionError> {
        let gpu = self.gpu.as_mut().unwrap();
        // let frame_buffer = self
        //     .system
        //     .memory
        //     .read(gpu.stdmem_frame_buffer_ptr, gpu.fb_size)?;
        let frame_buffer = &self.system.memory.physical[gpu.stdmem_frame_buffer_ptr as usize
            ..(gpu.stdmem_frame_buffer_ptr + gpu.fb_size) as usize];
        gpu.draw(frame_buffer)?;
        Ok(())
    }
    pub fn core_dump(&mut self) -> Result<(), ExecutionError> {
        const CORE: &str = "nisvc.core";
        let mut core_file = File::create(CORE)
            .map_err(|e| ExecutionError::new(format!("failed to dump core: {e}")))?;
        core_file
            .write_all(&self.system.memory.physical)
            .map_err(|e| ExecutionError::new(format!("failed to dump core: {e}")))?;
        println!("{}", "core dumped".on_red());
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
        let interface = self.get_interface(file_descriptor)?;
        interface.read(n)
    }
    fn write_file(&mut self, file_descriptor: u64, buffer: &[u8]) -> Result<(), ExecutionError> {
        let interface = self.get_interface(file_descriptor)?;
        interface.write(buffer)
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
        let mut buffer = Vec::with_capacity(n as usize);
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
            return Err(ExecutionError::new(format!("incomplete read from reader")));
        } else {
            Ok(buffer)
        }
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
}
