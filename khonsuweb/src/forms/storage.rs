use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct FormStorage<T>
where
    T: std::fmt::Debug + Default + Clone,
{
    value: Rc<RefCell<T>>,
    dirty: bool,
}

impl<T> FormStorage<T>
where
    T: std::fmt::Debug + Default + Clone + PartialEq,
{
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            dirty: false,
        }
    }

    pub fn update(&mut self, new_value: T) {
        self.dirty = self.dirty || self.value.borrow().eq(&new_value);
        *self.value.borrow_mut() = new_value;
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, T> {
        self.value.borrow()
    }

    pub fn value(&self) -> T {
        self.value.borrow().clone()
    }
}

impl<T> Into<Rc<RefCell<T>>> for FormStorage<T>
where
    T: std::fmt::Debug + Default + Clone + Eq,
{
    fn into(self) -> Rc<RefCell<T>> {
        self.value
    }
}

impl<T> FormStorage<T>
where
    T: Into<String> + std::fmt::Debug + Default + Clone + PartialEq,
{
    pub fn value_as_option(&self) -> Option<String> {
        let value = self.value().into();
        match value.len() {
            0 => None,
            _ => Some(value),
        }
    }
}
