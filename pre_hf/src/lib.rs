extern crate alloc;
extern crate core;
extern crate wee_alloc;

use alloc::vec::Vec;
use std::mem::MaybeUninit;
use std::slice;


// === 📝 log Host Function ===
#[link(wasm_import_module = "env")]
extern "C" {
  /// WebAssembly import which prints a string (linear memory offset,
  /// byteCount) to the console.
  ///
  /// Note: This is not an ownership transfer: Rust still owns the pointer
  /// and ensures it isn't deallocated during this call.
  #[link_name = "log"]
  fn _log(ptr: u32, size: u32);
}

pub fn log(message: &String) {
  unsafe {
      let (ptr, len) = string_to_ptr(message);
      _log(ptr, len);
  }
}



// === 🧰 Strings & Pointers Helpers ===
/// Returns a pointer and size pair for the given string in a way compatible
/// with WebAssembly numeric types.
///
/// Note: This doesn't change the ownership of the String. To intentionally
/// leak it, use [`std::mem::forget`] on the input after calling this.
pub unsafe fn string_to_ptr(s: &String) -> (u32, u32) {
  return (s.as_ptr() as u32, s.len() as u32);
}

/// Returns a string from WebAssembly compatible numeric types representing
/// its pointer and length.
pub unsafe fn ptr_to_string(ptr: u32, len: u32) -> String {
  let slice = slice::from_raw_parts_mut(ptr as *mut u8, len as usize);
  let utf8 = std::str::from_utf8_unchecked_mut(slice);
  return String::from(utf8);
}



// === 🧰 Memory Helpers ===


/// Set the global allocator to the WebAssembly optimized one.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// WebAssembly export that allocates a pointer (linear memory offset) that can
/// be used for a string.
///
/// This is an ownership transfer, which means the caller must call
/// [`deallocate`] when finished.
#[cfg_attr(all(target_arch = "wasm32"), export_name = "allocate")]
#[no_mangle]
pub extern "C" fn _allocate(size: u32) -> *mut u8 {
    allocate(size as usize)
}

/// Allocates size bytes and leaks the pointer where they start.
fn allocate(size: usize) -> *mut u8 {
    // Allocate the amount of bytes needed.
    let vec: Vec<MaybeUninit<u8>> = Vec::with_capacity(size);

    // into_raw leaks the memory to the caller.
    Box::into_raw(vec.into_boxed_slice()) as *mut u8
}

/// WebAssembly export that deallocates a pointer of the given size (linear
/// memory offset, byteCount) allocated by [`allocate`].
#[cfg_attr(all(target_arch = "wasm32"), export_name = "deallocate")]
#[no_mangle]
pub unsafe extern "C" fn _deallocate(ptr: u32, size: u32) {
    deallocate(ptr as *mut u8, size as usize);
}

/// Retakes the pointer which allows its memory to be freed.
unsafe fn deallocate(ptr: *mut u8, size: usize) {
    let _ = Vec::from_raw_parts(ptr, 0, size);
}





/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
*/
