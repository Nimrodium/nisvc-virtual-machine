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
pushc $!str
pushc $!len
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
# close
# runtime_silence_switch

# raw_tty_switch
# tty_rel_cursor
# tty_abs_cursor

# malloc
## C Notation
```c
void *malloc(uint64_t size)
```

# realloc
## C Notation
```c
void *realloc(void *ptr, uint64_t size)
```

# free
# memcpy
## C Notation
```c
void memcpy(void* dest,void* src, uint64_t n)
```

# memset
## C Notation
```c
void memset(void* dest,  uint8_t byte, uint64_t n)
```

# init_fb
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
> enumerated mode
>- 0
>> 8bpp greyscale ascii text mode
>- 1
>> 8bpp greyscale
>- 2
>> 8bpp rgb
>- 3
>> 24bpp rgb

# draw_fb
# get_fb_ptr
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
# video_mode_switch
switch kernel framebuffer rendering mode
## arguments
- mode
> enumerated switch
> - 0
>>raw bpp pixel data
> - 1
>>ascii text mode
# get_file_size
returns size of a file in bytes
## Arguments
- file_descriptor
## Returns
- file size in bytes

# dump
dump core to file in cwd on host

# kill
immediately kill execution