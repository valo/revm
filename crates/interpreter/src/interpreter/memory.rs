use crate::{alloc::vec::Vec, primitives::U256};
use core::{
    cmp::min,
    ops::{BitAnd, Not},
};

/// A sequential memory. It uses Rust's `Vec` for internal
/// representation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Memory {
    data: Vec<u8>,
}

impl Default for Memory {
    #[inline]
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(4 * 1024), // took it from evmone
        }
    }
}

impl Memory {
    /// Create a new memory with the given limit.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(4 * 1024), // took it from evmone
        }
    }

    #[deprecated = "Use `len` instead"]
    #[doc(hidden)]
    #[inline]
    pub fn effective_len(&self) -> usize {
        self.len()
    }

    /// Returns the length of the current memory range.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if current memory range length is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return a reference to the full memory.
    #[inline]
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    /// Consumes the type and returns the full memory.
    #[inline]
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    /// Shrinks the capacity of the data buffer as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit()
    }

    /// Resizes the stack in-place so that then length is equal to `new_size`.
    ///
    /// `new_size` should be a multiple of 32.
    #[inline]
    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0);
    }

    /// Returns a byte slice of the memory region at the given offset.
    #[inline]
    pub fn get_slice(&self, offset: usize, size: usize) -> &[u8] {
        &self.data[offset..offset + size]
    }

    /// Sets the `byte` at the given `index`.
    #[inline]
    pub fn set_byte(&mut self, index: usize, byte: u8) {
        self.data[index] = byte;
    }

    /// Sets the given `value` to the memory region at the given `offset`.
    #[inline]
    pub fn set_u256(&mut self, index: usize, value: U256) {
        self.data[index..index + 32].copy_from_slice(&value.to_be_bytes::<{ U256::BYTES }>());
    }

    /// Set memory region at given `offset`.
    #[inline]
    pub fn set(&mut self, offset: usize, value: &[u8]) {
        if !value.is_empty() {
            self.data[offset..(value.len() + offset)].copy_from_slice(value);
        }
    }

    /// Set memory from data. Our memory offset+len is expected to be correct but we
    /// are doing bound checks on data/data_offeset/len and zeroing parts that is not copied.
    #[inline]
    pub fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]) {
        if data_offset >= data.len() {
            // nullify all memory slots
            for i in &mut self.data[memory_offset..memory_offset + len] {
                *i = 0;
            }
            return;
        }
        let data_end = min(data_offset + len, data.len());
        let memory_data_end = memory_offset + (data_end - data_offset);
        self.data[memory_offset..memory_data_end].copy_from_slice(&data[data_offset..data_end]);

        // nullify rest of memory slots
        // Safety: Memory is assumed to be valid. And it is commented where that assumption is made
        for i in &mut self.data[memory_data_end..memory_offset + len] {
            *i = 0;
        }
    }

    /// In memory copy given a src, dst, and length
    #[inline(always)]
    pub fn copy(&mut self, dst: usize, src: usize, len: usize) {
        self.data.copy_within(src..src + len, dst);
    }
}

/// Rounds up `x` to the closest multiple of 32. If `x % 32 == 0` then `x` is returned.
#[inline]
pub(crate) fn next_multiple_of_32(x: usize) -> Option<usize> {
    let r = x.bitand(31).not().wrapping_add(1).bitand(31);
    x.checked_add(r)
}

#[cfg(test)]
mod tests {
    use super::next_multiple_of_32;
    use crate::Memory;

    #[test]
    fn test_copy() {
        // Create a sample memory instance
        let mut memory = Memory::new();

        // Set up initial memory data
        let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        memory.resize(data.len());
        memory.set_data(0, 0, data.len(), &data);

        // Perform a copy operation
        memory.copy(5, 0, 4);

        // Verify the copied data
        let copied_data = memory.get_slice(5, 4);
        assert_eq!(copied_data, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_next_multiple_of_32() {
        // next_multiple_of_32 returns x when it is a multiple of 32
        for i in 0..32 {
            let x = i * 32;
            assert_eq!(Some(x), next_multiple_of_32(x));
        }

        // next_multiple_of_32 rounds up to the nearest multiple of 32 when `x % 32 != 0`
        for x in 0..1024 {
            if x % 32 == 0 {
                continue;
            }
            let next_multiple = x + 32 - (x % 32);
            assert_eq!(Some(next_multiple), next_multiple_of_32(x));
        }
    }
}
