use std::marker::PhantomData;
use std::ptr::NonNull;

#[repr(transparent)]
pub struct Local<'sc, T> {
  value: NonNull<T>,
  scope: PhantomData<&'sc ()>,
}

impl<'sc, T> Local<'sc, T> {
  pub fn from_raw<S: 'sc>(_scope: &mut S, ptr: *mut T) -> Option<Self> {
    NonNull::new(ptr).map(|value| Self {
      value,
      scope: PhantomData,
    })
  }
}
