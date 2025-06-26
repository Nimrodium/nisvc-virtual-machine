# NHK Syscalls
- 0x01 **[open(2)](#open)**
- 0x02 **[write(3)](#write)**
- 0x03 **[read(3)](#read)**
- 0x04 **[seek(3)](#seek)**
- 0x05 **[close(1)](#close)**
- 0x06 **[runtime_silence_switch(1)](#runtime_silence_switch)**
- 0x07 **[raw_tty_switch(1)](#raw_tty_switch)**
- 0x08 **[tty_rel_cursor(2)](#tty_rel_cursor)**
- 0x09 **[tty_abs_cursor(2)](#tty_abs_cursor)**
- 0x0a **[malloc(1)](#malloc)**
- 0x0b **[realloc(2)](#realloc)**
- 0x0c **[free(1)](#free)**
- 0x0d **[memcpy(3)](#memcpy)**
- 0x0e **[memset(3)](#memset)**
- 0x0f **[init_fb(4)](#init_fb)**
- 0x10 **[draw_fb(0)](#draw_fb)**
- 0x11 **[get_fb_ptr(0)](#get_fb_ptr)**
- 0x12 **[get_file_size(1)](#get_file_size)**
- 0x13 **[dump(0)](#dump)**
- 0x14 **[kill(0)](#kill)**
- 0x15 **[get_argc(0)](#get_argc)**
- 0x16 **[get_argv(1)](#get_argv)**
# open
1Interrupt Code: `0x01`
## C notation
```c
int open(int str_ptr, int str_len);
```
opens a file from the host fs from the file path string
## arguments
- str_ptr
> pointer to start of path string
- str_len
> length of string
## returns
- file_descriptor
> integer used to specify the file to interface with for other syscalls

## example
```asm
pushi $!str
pushi $!len
int $x01
pop r1 # file_descriptor
```

# write
Interrupt Code: 0x2
## C Notation
```c
void write(uint64_t fd, void* buffer, void* n);
```

# read
Interrupt Code: 0x3
```c
void read(uint64_t fd, void* buffer, void* n);
```

# seek
Interrupt Code: 0x4
# close
Interrupt Code: 0x5
# runtime_silence_switch
Interrupt Code: 0x6
# raw_tty_switch
Interrupt Code: 0x7
# tty_rel_cursor
Interrupt Code: 0x8
# tty_abs_cursor
Interrupt Code: 0x9
# malloc
Interrupt Code: 0xa
## C Notation
```c
void *malloc(uint64_t size)
```

# realloc
Interrupt Code: 0xb
## C Notation
```c
void *realloc(void *ptr, uint64_t size)
```

# free
Interrupt Code: 0xc
# memcpy
Interrupt Code: 0xd
## C Notation
```c
void memcpy(void* dest,void* src, uint64_t n)
```

# memset
Interrupt Code: 0xe
## C Notation
```c
void memset(void* dest,  uint8_t byte, uint64_t n)
```

# init_fb
Interrupt Code: 0xf
## C notation

```c
void init_fb(int frame_buffer_ptr, int width, int height, int mode);
```

## arguments

- frame_buffer
> pointer to start of userspace framebuffer
- width
> width of display in bytes
- height
> height of display in bytes
- mode


# draw_fb
Interrupt Code: 0x10
# get_fb_ptr
Interrupt Code: 0x11
get pointer to start of the framebuffer
## returns
- fb_ptr
> pointer to framebuffer start
## example
```asm
int $x11
pop r1 # fb_ptr
ldi r2b1,$1
store r1,r2b1,r2b1 # write top pixel as 1
int $x10 #refresh
```

# get_file_size
Interrupt Code: 0x12
returns size of a file in bytes
## Arguments
- file_descriptor
## Returns
- file size in bytes

# dump
Interrupt Code: 0x13
dump core to file in cwd on host

# kill
Interrupt Code: 0x14
immediately kill execution

# get_argc
Interrupt Code: 0x15
get argc; number of arguments in cmdline

# get_argv
Interrupt Code: 0x16
get argument from cmdline at index
