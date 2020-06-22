use super::{ValidationError, Validator};
use crate::forms::storage::FormStorage;

pub trait Presentable: Default + Clone + PartialEq {
    fn present(&self) -> bool;

    fn absent(&self) -> bool {
        !self.present()
    }
}

impl Presentable for String {
    fn present(&self) -> bool {
        !self.is_empty()
    }
}

impl Presentable for u8 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for u16 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for u32 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for u64 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for u128 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for i8 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for i16 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for i32 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for i64 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for i128 {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for isize {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for usize {
    fn present(&self) -> bool {
        self != &0
    }
}

impl Presentable for f32 {
    fn present(&self) -> bool {
        self.is_finite() && float_cmp::approx_eq!(f32, *self, 0.0)
    }
}

impl Presentable for f64 {
    fn present(&self) -> bool {
        self.is_finite() && float_cmp::approx_eq!(f64, *self, 0.0)
    }
}

impl<T> Presentable for Option<T>
where
    T: Presentable,
{
    fn present(&self) -> bool {
        match self {
            Some(p) => p.present(),
            None => false,
        }
    }
}

#[derive(Debug)]
pub struct PresentValidation<T>
where
    T: Default + Clone + PartialEq + std::fmt::Debug,
{
    pub value: FormStorage<T>,
}

impl<T> Validator for PresentValidation<T>
where
    T: Presentable + std::fmt::Debug,
{
    fn validate(&self) -> Result<(), ValidationError> {
        if self.value.value()?.present() {
            Ok(())
        } else {
            Err(ValidationError::NotPresent)
        }
    }
}

#[derive(Debug)]
pub struct AbsentValidation<T>
where
    T: Default + Clone + PartialEq + std::fmt::Debug,
{
    pub value: FormStorage<T>,
}

impl<T> Validator for AbsentValidation<T>
where
    T: Presentable + std::fmt::Debug,
{
    fn validate(&self) -> Result<(), ValidationError> {
        if self.value.value()?.absent() {
            Ok(())
        } else {
            Err(ValidationError::NotAbsent)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::validations::prelude::*;

    #[test]
    fn present_strings() {
        String::new()
            .is_present()
            .validate()
            .expect_err("New string should not be considered present");
        String::from("value")
            .is_present()
            .validate()
            .expect("Value should be considered present");
    }

    #[test]
    fn present_unsigned_ints() {
        0u8.is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1u8.is_present()
            .validate()
            .expect("Value should be considered present");
        0u16.is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1u16.is_present()
            .validate()
            .expect("Value should be considered present");
        0u32.is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1u32.is_present()
            .validate()
            .expect("Value should be considered present");
        0u64.is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1u64.is_present()
            .validate()
            .expect("Value should be considered present");
        0u128
            .is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1u128
            .is_present()
            .validate()
            .expect("Value should be considered present");
        0usize
            .is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1usize
            .is_present()
            .validate()
            .expect("Value should be considered present");
    }

    #[test]
    fn present_signed_ints() {
        0i8.is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1i8.is_present()
            .validate()
            .expect("Value should be considered present");
        0i16.is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1i16.is_present()
            .validate()
            .expect("Value should be considered present");
        0i32.is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1i32.is_present()
            .validate()
            .expect("Value should be considered present");
        0i64.is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1i64.is_present()
            .validate()
            .expect("Value should be considered present");
        0i128
            .is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1i128
            .is_present()
            .validate()
            .expect("Value should be considered present");
        0isize
            .is_present()
            .validate()
            .expect_err("New string should not be considered present");
        1isize
            .is_present()
            .validate()
            .expect("Value should be considered present");
    }
}
