use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};
use thiserror::Error;
pub mod combinators;
pub mod present;
use combinators::*;
use present::*;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("is required")]
    NotPresent,
}

pub trait Validator: std::fmt::Debug {
    fn validate(&self) -> Result<(), ValidationError>;
}

pub trait ValidatorCombinators: Sized + Validator {
    fn and<U: Validator>(self, other: U) -> AndValidation<Self, U> {
        AndValidation {
            left: self,
            right: other,
        }
    }

    fn or<U: Validator>(self, other: U) -> OrValidation<Self, U> {
        OrValidation {
            left: self,
            right: other,
        }
    }
}

pub trait Validatable<T>
where
    T: std::fmt::Debug,
{
    fn is_present(&self) -> PresentValidation<T>;
}

impl<T> Validatable<T> for T
where
    T: Presentable + Clone + std::fmt::Debug,
{
    fn is_present(&self) -> PresentValidation<T> {
        PresentValidation {
            value: Rc::new(RefCell::new(self.clone())),
        }
    }
}

impl<T> Validatable<T> for Rc<RefCell<T>>
where
    T: Presentable + std::fmt::Debug,
{
    fn is_present(&self) -> PresentValidation<T> {
        PresentValidation {
            value: self.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Feild<F, V>
where
    F: std::fmt::Debug,
    V: std::fmt::Debug,
{
    field: F,
    value: V,
}

#[derive(Error, Debug)]
pub struct FieldError<F>
where
    F: std::fmt::Debug,
{
    fields: HashSet<F>,
    #[source]
    error: ValidationError,
}

impl<F> std::fmt::Display for FieldError<F>
where
    F: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "FieldError {{ fields: {:?}, error: {:?} }}",
            self.fields, self.error
        ))
    }
}

#[derive(Error, Debug)]
pub struct ErrorSet<F>
where
    F: std::fmt::Debug,
{
    errors: HashMap<F, Vec<Rc<FieldError<F>>>>,
}

impl<F> ErrorSet<F>
where
    F: Copy + std::fmt::Debug + std::hash::Hash + std::cmp::Eq,
{
    pub fn errors_for(&self, field: &F) -> Option<Vec<Rc<FieldError<F>>>> {
        self.errors.get(field).map(|errors| errors.clone())
    }
}

impl<F> std::fmt::Display for ErrorSet<F>
where
    F: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ErrorSet {{ errors: {:?} }}", self.errors))
    }
}

pub struct ModelValidator<F>
where
    F: std::fmt::Debug + std::hash::Hash + std::cmp::Eq,
{
    validations: HashMap<F, Box<dyn Validator>>,
}

impl<F> ModelValidator<F>
where
    F: Copy + std::fmt::Debug + std::hash::Hash + std::cmp::Eq,
{
    pub fn new() -> Self {
        Self {
            validations: HashMap::new(),
        }
    }
    pub fn with_field<V: Validator + 'static>(mut self, field: F, validator: V) -> Self {
        self.validations.insert(field, Box::new(validator));
        self
    }
    pub fn validate(self) -> Option<Rc<ErrorSet<F>>> {
        let mut errors = HashMap::new();
        for (field, validation) in self.validations.into_iter() {
            if let Err(error) = validation.validate() {
                let mut fields = HashSet::new();
                fields.insert(field);
                let error = Rc::new(FieldError { fields, error });
                errors.insert(field, vec![error]);
            }
        }

        if errors.len() > 0 {
            Some(Rc::new(ErrorSet { errors }))
        } else {
            None
        }
    }
}

pub mod prelude {
    pub use super::combinators::*;
    pub use super::present::*;
    pub use super::{ErrorSet, ModelValidator, Validatable, ValidationError, Validator};
}
