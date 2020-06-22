use crate::validations::{ValidationError, Validator};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct FormStorage<T>
where
    T: std::fmt::Debug + Default + Clone,
{
    value: Rc<RefCell<FormStorageBacking<T>>>,
}

#[derive(Debug, Default, Clone)]
struct FormStorageBacking<T>
where
    T: std::fmt::Debug + Default + Clone,
{
    value: T,
    dirty: bool,
    invalid_value: bool,
}

impl<T> FormStorage<T>
where
    T: std::fmt::Debug + Default + Clone + PartialEq,
{
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(FormStorageBacking {
                value,
                dirty: false,
                invalid_value: false,
            })),
        }
    }

    pub fn update(&mut self, new_value: T) {
        let mut backing = self.value.borrow_mut();
        backing.dirty = backing.dirty || backing.value.eq(&new_value);
        backing.value = new_value;
    }

    pub fn update_with_invalid_hint(&mut self, new_value: T, invalid: bool) {
        self.update(new_value);
        self.value.borrow_mut().invalid_value = invalid;
    }

    pub fn update_invalid_hint(&mut self, invalid: bool) {
        self.value.borrow_mut().invalid_value = invalid;
    }

    pub fn value(&self) -> Result<T, ValidationError> {
        self.validate().map(|_| self.unchecked_value())
    }

    pub fn unchecked_value(&self) -> T {
        self.value.borrow().value.clone()
    }
}

impl<T> Validator for FormStorage<T>
where
    T: std::fmt::Debug + Default + Clone + PartialEq,
{
    fn validate(&self) -> Result<(), ValidationError> {
        let backing = self.value.borrow();

        if backing.invalid_value {
            return Err(ValidationError::InvalidValue);
        }
        Ok(())
    }
}
