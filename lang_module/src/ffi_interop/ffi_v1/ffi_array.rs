use crate::ffi_interop::ffi_v1::FFIType;

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use std::mem;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::slice::from_raw_parts_mut;

#[repr(C)]
pub struct FFIArray<T: FFIType> {
    entries: *mut T,
    size: usize,
}
impl<T: FFIType> AsRef<[T]> for FFIArray<T> {
    fn as_ref(&self) -> &[T] {
        let entries = unsafe { from_raw_parts_mut(self.entries, self.size) };
        entries
    }
}
impl<T: FFIType + Serialize> Serialize for FFIArray<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let values = unsafe { from_raw_parts_mut(self.entries, self.size) };
        let mut seq = serializer.serialize_seq(Some(self.size))?;
        for value in values.iter() {
            seq.serialize_element(value)?;
        }
        seq.end()
    }
}
impl<T: FFIType + Debug> Debug for FFIArray<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut entries = f.debug_list();
        let values = unsafe { from_raw_parts_mut(self.entries, self.size) };
        for entry in values.iter() {
            entries.entry(entry);
        }
        entries.finish()
    }
}
impl<T: FFIType + Clone> Clone for FFIArray<T> {
    fn clone(&self) -> Self {
        let mut entries = Vec::with_capacity(self.size);
        let values = unsafe { from_raw_parts_mut(self.entries, self.size) };
        for value in values.iter() {
            entries.push(value.clone());
        }
        let mut entries_box = entries.into_boxed_slice();
        let entries = entries_box.as_mut_ptr();
        mem::forget(entries_box);
        Self {
            entries,
            size: self.size,
        }
    }
}

impl<T: FFIType + PartialEq> PartialEq for FFIArray<T> {
    fn eq(&self, other: &Self) -> bool {
        let Self { entries, size } = self;
        let Self {
            entries: other_entries,
            size: other_size,
        } = other;
        if size != other_size {
            return false;
        }
        let entries = unsafe { from_raw_parts_mut(*entries, *size) };
        let other_entries = unsafe { from_raw_parts_mut(*other_entries, *other_size) };
        let mut result = true;
        for (entry, other_entry) in entries.iter().zip(other_entries.iter()) {
            result &= entry == other_entry;
        }
        result
    }
}

impl<T: FFIType> Default for FFIArray<T> {
    fn default() -> Self {
        Self {
            entries: std::ptr::null_mut(),
            size: 0,
        }
    }
}
impl<T: FFIType> TryFrom<Vec<T::SafeType>> for FFIArray<T> {
    type Error = <T::SafeType as TryInto<T>>::Error;

    fn try_from(value: Vec<T::SafeType>) -> Result<Self, Self::Error> {
        let size = value.len();
        let mut raw_entries = Vec::with_capacity(size);
        for value in value {
            match value.try_into() {
                Ok(ok) => raw_entries.push(ok),
                Err(err) => {
                    // Drop all the values we've already converted
                    for value in raw_entries {
                        drop(value)
                    }
                    return Err(err);
                }
            }
        }
        let mut entries = raw_entries.into_boxed_slice();
        let entries_ptr = entries.as_mut_ptr();
        mem::forget(entries);
        Ok(Self {
            entries: entries_ptr,
            size,
        })
    }
}
impl<T: FFIType> TryFrom<FFIArray<T>> for Vec<T::SafeType> {
    type Error = <T::SafeType as TryFrom<T>>::Error;

    fn try_from(value: FFIArray<T>) -> Result<Self, Self::Error> {
        let mut result: Vec<T::SafeType> = Vec::with_capacity(value.size);
        let mut error: Option<Self::Error> = None;
        for entry in value.into_iter() {
            match entry.try_into() {
                Ok(ok) => result.push(ok),
                Err(err) => {
                    // Yes if multiple values error out only the first one is returned.
                    // But all values need to be dropped safely and I dont want to allocate a Vec to store the errors.
                    if error.is_none() {
                        error = Some(err);
                    }
                }
            }
        }
        // All types were converted to the Safe Type or Errored Out.
        // Hopefully the type that errored safely dropped itself.
        if let Some(err) = error {
            return Err(err);
        }

        Ok(result)
    }
}
impl<T: FFIType> Drop for FFIArray<T> {
    fn drop(&mut self) {
        if self.entries.is_null() {
            return;
        }
        unsafe {
            let s = from_raw_parts_mut(self.entries, self.size);
            let raw_entries = Box::from_raw(s);
            let drop = raw_entries.into_vec();
            for value in drop.into_iter() {
                mem::drop(value)
            }
        }
    }
}
impl<T: FFIType> IntoIterator for FFIArray<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        if self.size == 0 {
            assert!(self.entries.is_null());
            return vec![].into_iter();
        }
        let manual_drop = ManuallyDrop::new(self);
        let entries = manual_drop.entries;
        let size = manual_drop.size;

        unsafe {
            let s = from_raw_parts_mut(entries, size);
            let raw_entries = Box::from_raw(s);
            raw_entries.into_vec().into_iter()
        }
    }
}

#[derive(Debug)]
pub struct FFIArrayInner<T: FFIType>(ManuallyDrop<Box<[T]>>, usize);
impl<T: FFIType> Drop for FFIArrayInner<T> {
    fn drop(&mut self) {
        if self.1 == 0 {
            unsafe { ManuallyDrop::drop(&mut self.0) }
        }
    }
}
impl<T: FFIType> Deref for FFIArrayInner<T> {
    type Target = Box<[T]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: FFIType> DerefMut for FFIArrayInner<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: FFIType> AsRef<[T]> for FFIArrayInner<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}
impl<T: FFIType> FFIArray<T> {
    /// # Safety
    /// The Box within the FFIArrayInner can not be. Data within the Box must stay the same size
    pub fn as_box(&self) -> FFIArrayInner<T> {
        unsafe {
            let s = from_raw_parts_mut(self.entries, self.size);
            FFIArrayInner(ManuallyDrop::new(Box::from_raw(s)), self.size)
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    /// Prints the raw values of the FFIArray.
    pub fn raw_debug(&self) -> impl Debug {
        format!("{:?} {:?}", self.entries, self.size)
    }
    /// # Safety
    /// You are now responsible deallocating the memory once used.
    pub unsafe fn into_inner(self) -> (*mut T, usize) {
        let manual_drop = ManuallyDrop::new(self);
        let entries = manual_drop.entries;
        let size = manual_drop.size;
        (entries, size)
    }
}
#[cfg(test)]
mod tests {
    use crate::ffi_interop::ffi_v1::ffi_array::FFIArray;
    use crate::ffi_interop::ffi_v1::FFIString;

    fn test_string_vec() -> Vec<String> {
        let mut vec = Vec::with_capacity(20);
        for i in 0..20 {
            vec.push(format!("test {}", i));
        }
        vec
    }

    fn test_usize_vec() -> Vec<usize> {
        let mut vec = Vec::with_capacity(20);
        for i in 0..20 {
            vec.push(i);
        }
        vec
    }

    #[test]
    pub fn test_build() {
        let _ffi_array: FFIArray<FFIString> = test_string_vec().try_into().unwrap();
    }

    #[test]
    pub fn test_into_iter() {
        let ffi_array: FFIArray<FFIString> = test_string_vec().try_into().unwrap();
        for (i, entry) in ffi_array.into_iter().enumerate() {
            println!("{}: {:?}", i, entry)
        }
    }
    #[test]
    pub fn test_to_vec() {
        {
            let ffi_array: FFIArray<usize> = test_usize_vec().try_into().unwrap();
            let vec: Vec<usize> = ffi_array.try_into().unwrap();
            assert_eq!(vec, test_usize_vec());
        }
        {
            let ffi_array: FFIArray<FFIString> = test_string_vec().try_into().unwrap();
            let vec: Vec<String> = ffi_array.try_into().unwrap();
            assert_eq!(vec, test_string_vec());
        }
    }

    #[test]
    pub fn null_tests() {
        let ffi_array: FFIArray<FFIString> = Default::default();
        assert_eq!(ffi_array.len(), 0);
        assert!(ffi_array.is_empty());
        let vec: Vec<String> = ffi_array.try_into().unwrap();
        assert!(vec.is_empty());
    }
    #[test]
    pub fn test_clone() {
        let ffi_array: FFIArray<FFIString> = test_string_vec().try_into().unwrap();
        let clone = ffi_array.clone();
        assert_eq!(clone, ffi_array);
    }
    #[test]
    pub fn debug() {
        let ffi_array: FFIArray<FFIString> = test_string_vec().try_into().unwrap();
        println!("{:?}", ffi_array);
        println!("{:?}", ffi_array.raw_debug());
    }
}
