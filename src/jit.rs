#![allow(dead_code)]
use libc::{ PROT_EXEC, PROT_READ, PROT_WRITE, c_void, mprotect, munmap, posix_memalign};
use std::ops::{Drop, Fn};

use crate::parser::Ops;

const PAGESIZE: usize = 4096;

#[repr(C)]
#[derive(Debug)]
pub struct Context {
    output: Vec<u8>
}

impl Context {
    pub fn new(output_size: usize) -> Context {
        Context {
            output: vec![0; output_size]
        }
    }

    pub fn to_string(&self) -> String {
        self.output.iter().map(|b| char::from(*b)).collect::<String>()
    }
}

pub struct JITFn {
    addr: *mut u8,
    size_pages: usize,
    size_bytes: isize
}

impl Fn<(*mut Context,)> for JITFn {
    extern "rust-call" fn call(&self, args: (*mut Context,)) {
        unsafe {
            let fn_ptr: extern "C" fn(*mut Context);

            fn_ptr = std::mem::transmute(self.addr);

            fn_ptr(args.0);
        }
    }
}

impl FnOnce<(*mut Context,)> for JITFn {
    type Output = ();
    extern "rust-call" fn call_once(self, args: (*mut Context,)) {
        self.call(args)
    }
}

impl FnMut<(*mut Context,)> for JITFn {
    extern "rust-call" fn call_mut(&mut self, args: (*mut Context,)) {
        self.call(args)
    }
}

impl Drop for JITFn {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(munmap(self.addr as *mut c_void, self.size_pages * PAGESIZE as usize), 0);
        }
    }
}

// x86_64 instruction encoding

const RET: u8 = 0xc3;

const PUSH_RBP: u8 = 0x55;

const PUSH_RAX: [u8; 1] = [0x50];

const MOV_RBP_RSP: [u8; 3] = [0x48, 0x89, 0xec];

const MOV_RSP_RBP: [u8; 3] = [0x48, 0x89, 0xe5];

const MOV_R8_RBP: [u8; 3] = [0x48, 0x89, 0xe8];

const POP_RBP: u8 = 0x5D;

const XOR_R8_R8: [u8; 3] = [0x4d, 0x31, 0xc0]; // Zero r8

const INC_R8: [u8; 3] = [0x49, 0xff, 0xc0];

const DEC_R8: [u8; 3] = [0x49, 0xff, 0xc8];

const INC_R9: [u8; 3] = [0x49, 0xff, 0xc1];

const XOR_RAX_RAX: [u8; 3] = [0x48, 0x31, 0xc0];

// INC byte [rsp + r8]
const INC_VALUE: [u8; 4] = [0x42, 0xfe, 0x04, 0x04];

// DEC byte [rsp + r8]
const DEC_VALUE: [u8; 4] = [0x42, 0xfe, 0x0c, 0x04];

// ADD RSP Immediate
const ADD_RSP_32: [u8; 4] = [0x48, 0x83, 0xc4, 0x20];

// SUB RSP Immediate
const SUB_RSP_32: [u8; 4] = [0x48, 0x83, 0xec, 0x20];

// MOV R9 RSI
const MOV_R9_RSI: [u8; 3] = [0x4c, 0x8b, 0x0e];

// MOV RAX, [RBP + R8]
//const MOV_RAX_R8_RBP: [u8; 5] = [0x4a, 0x8b, 0x44, 0x05, 00];

// MOV RAX (AL), byte [RSP + R8]
const MOV_RAX_R8_RSP: [u8; 4] = [0x42, 0x8a, 0x04, 0x04];

// MOV byte [R9], RAX (AL)
const MOV_R9_RAX: [u8; 3] = [0x41, 0x88, 0x01];

// CMP byte [RBP + r8], $0
const CMP_RSP_R8_0: [u8; 5] = [0x42, 0x80, 0x3c, 0x04, 0x00];

// This trait allows us to define an assembler that uses predefined instruction encoding
trait JITAssembler {
    fn push_u8(&mut self, value: u8);

    fn current_addr(&self) -> usize;

    fn push_bytes(&mut self, values: &[u8]) {
        for v in values.into_iter() {
            self.push_u8(*v);
        }
    }

    fn assemble(&mut self, ops: &Vec<Ops>) {
        self.prologue();

        self.binary_translation(ops.as_slice());

        self.epilogue();

        self.ret();

        println!("Buffer filled with instructions");
    }

    fn prologue(&mut self) {
        // sysv abi
        self.push_rbp();
        self.mov_rbp_rsp();

        self.xor_rax_rax(); // Zero RAX (pushed on the stack to zero 8 bytes)
        self.sub_rsp_32(); // Allocate 32 bytes on the stack
        self.xor_r8_r8(); // Zero data offset (data pointer)
        self.mov_r9_rsi(); // Initialize r9 (pointer to output struct)
    }

    fn epilogue(&mut self) {
        self.add_rsp_32();

        // sysv abi
        self.mov_rsp_rbp();
        self.pop_rbp();
    }

    fn mov_r9_rsi(&mut self) {
        self.push_bytes(&MOV_R9_RSI);
    }

    fn xor_r8_r8(&mut self) {
        self.push_bytes(&XOR_R8_R8);
    }

    fn ret(&mut self) {
        self.push_u8(RET);
    }

    fn add_rsp_32(&mut self) {
        self.push_bytes(&ADD_RSP_32);
    }

    fn sub_rsp_32(&mut self) {
        //self.push_bytes(&SUB_RSP_32);
        self.push_bytes(&PUSH_RAX);
        self.push_bytes(&PUSH_RAX);
        self.push_bytes(&PUSH_RAX);
        self.push_bytes(&PUSH_RAX);
    }

    fn xor_rax_rax(&mut self) {
        self.push_bytes(&XOR_RAX_RAX);
    }

    // These instructions are used for the System V ABI

    fn pop_rbp(&mut self) {
        self.push_u8(POP_RBP);
    }

    fn push_rbp(&mut self) {
        self.push_u8(PUSH_RBP);
    }

    fn mov_rsp_rbp(&mut self) {
        self.push_bytes(&MOV_RSP_RBP);
    }

    fn mov_rbp_rsp(&mut self) {
        self.push_bytes(&MOV_RBP_RSP);
    }

    fn binary_translation(&mut self, ops: &[Ops]) {

        let mut loop_entry = Vec::new();

        for o in ops {
            match *o {
                Ops::IncrementPtr => self.push_bytes(&INC_R8),
                Ops::DecrementPtr => self.push_bytes(&DEC_R8),
                Ops::Increment => self.push_bytes(&INC_VALUE),
                Ops::Decrement => self.push_bytes(&DEC_VALUE),
                Ops::PutChar => {
                    self.push_bytes(&MOV_RAX_R8_RSP);
                    self.push_bytes(&MOV_R9_RAX);
                    self.push_bytes(&INC_R9);
                },
                Ops::OpenLoop => {
                    loop_entry.push(self.current_addr());
                },
                Ops::CloseLoop => {
                    self.push_bytes(&CMP_RSP_R8_0);
                    // JNZ rel16
                    self.push_bytes(&[0x0f, 0x85]);
                    let entry = loop_entry.pop().expect("Attempted to close loop that was never opened");
                    let current = self.current_addr();
                    let offset = (current + std::mem::size_of::<i32>() ) - entry;
                    println!("Offset: {}", offset);
                    self.push_bytes(&(-1 * (offset as i32)).to_le_bytes());
                },
                _ => ()
            };
        }
    }
}

enum Protection {
    ReadWrite,
    Executable
}

struct JITBuffer {
    addr: *mut u8,
    offset: isize,
    size: usize,
    prot: Protection,
}

impl JITBuffer {
    pub unsafe fn new(size: usize) -> JITBuffer {
        let mut ptr: *mut c_void = std::ptr::null_mut();

        let rw = PROT_READ | PROT_WRITE;
 
        assert_eq!(posix_memalign(&mut ptr, PAGESIZE as usize, size * PAGESIZE as usize), 0);

        mprotect(ptr, size * PAGESIZE as usize, rw);

        JITBuffer {
            addr: ptr as *mut u8,
            offset: 0,
            size,
            prot: Protection::ReadWrite
        }
    }

    fn to_jit_fn(mut self) -> JITFn {
        self.map_executable();
        JITFn {
            addr: self.addr,
            size_pages: self.size,
            size_bytes: self.offset
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for i in 0..self.offset {
            unsafe {
                let b = *self.addr.offset(i);
                bytes.push(b);
            }
        }
        bytes
    }

    fn impl_push_u8(&mut self, value: u8) {
        unsafe {
            *self.addr.offset(self.offset) = value;
            self.offset += 1;
        }
    }

    fn map_executable(&mut self) {
        match self.prot {
            Protection::Executable => (),
            Protection::ReadWrite => {
                unsafe {
                    assert_eq!(mprotect(self.addr as *mut c_void, self.size * PAGESIZE as usize, PROT_EXEC), 0);
                }
            }
        }
    }
}

impl JITAssembler for JITBuffer {
    fn push_u8(&mut self, value: u8) {
        self.impl_push_u8(value)
    }

    fn current_addr(&self) -> usize {
        unsafe {
            self.addr.offset(self.offset) as usize
        }
    }
}


pub fn jit_compile(input: &Vec<Ops>) -> JITFn {
    let mut buffer = unsafe { JITBuffer::new(1) };
    buffer.assemble(input);

    buffer.to_jit_fn()
}

pub fn jit_compile_to_bytes(input: &Vec<Ops>) -> Vec<u8> {
    let mut buffer = unsafe { JITBuffer::new(1) };
    buffer.assemble(input);
    buffer.bytes()
}

#[inline]
fn print_regs() {
    let mut r8: u64;
    let mut r9: u64;

    unsafe {
        asm!("mov {}, r8", out(reg) r8);
        asm!("mov {}, r9", out(reg) r9);
    }

    println!("r8: {}", r8);
    println!("r9: {}", r9);
}
