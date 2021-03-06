#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(try_trait)]
#![no_std]

extern crate alloc;
extern crate uefi;

use alloc::heap::{Alloc, AllocErr, Layout};
use core::ops::Try;
use uefi::memory::MemoryType;
use uefi::system::SystemTable;

static mut UEFI: *mut SystemTable = 0 as *mut SystemTable;

pub unsafe fn init(table: &'static mut SystemTable) {
    UEFI = table;
}

fn get_uefi() -> Option<&'static mut SystemTable> {
    unsafe {
        if UEFI as usize == 0 {
            None
        } else {
            Some(&mut *UEFI)
        }
    }
}

pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if let Some(ref mut uefi) = get_uefi() {
            let mut ptr = 0;
            if let Err(_) = (uefi.BootServices.AllocatePool)(MemoryType::EfiLoaderData, layout.size(), &mut ptr).into_result() {
                Err(AllocErr::Exhausted {
                    request: layout
                })
            } else {
                Ok(ptr as *mut u8)
            }
        } else {
            panic!("__rust_allocate: uefi not initialized");
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        if let Some(ref mut uefi) = get_uefi() {
            let _ = (uefi.BootServices.FreePool)(ptr as usize);
        } else {
            panic!("__rust_deallocate: uefi not initialized");
        }
    }
}
