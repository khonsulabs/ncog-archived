use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};
use thiserror::Error;
pub mod combinators;
pub mod present;
use crate::forms::storage::FormStorage;
use combinators::*;
use present::*;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("is required")]
    NotPresent,
    /// For when converting from a string to another type fails. Should be validated in another way.
    #[error("invalid value")]
    InvalidValue,
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
    T: Clone + Default + PartialEq + std::fmt::Debug,
{
    fn is_present(&self) -> PresentValidation<T>;
}

impl<T> Validatable<T> for T
where
    T: Presentable + Clone + std::fmt::Debug,
{
    fn is_present(&self) -> PresentValidation<T> {
        PresentValidation {
            value: FormStorage::new(self.clone()),
        }
    }
}

impl<T> Validatable<T> for FormStorage<T>
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
    F: Copy + std::fmt::Debug,
    V: std::fmt::Debug,
{
    field: F,
    value: V,
}

#[derive(Error, Debug)]
pub struct FieldError<F>
where
    F: Copy + std::fmt::Debug,
{
    pub fields: HashSet<F>,
    #[source]
    pub error: ValidationError,
}

impl<F> FieldError<F>
where
    F: Copy + std::fmt::Debug,
{
    pub fn primary_field(&self) -> F {
        *self.fields.iter().next().expect("No fields on FieldError")
    }
}

impl<F> std::fmt::Display for FieldError<F>
where
    F: Copy + std::fmt::Debug,
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
    F: Copy + std::fmt::Debug,
{
    errors: Vec<FieldError<F>>,
}

impl<F> ErrorSet<F>
where
    F: Copy + std::fmt::Debug + std::hash::Hash + std::cmp::Eq,
{
    pub fn translate<T, S>(&self, translator: T) -> Rc<HashMap<F, Vec<Rc<yew::Html>>>>
    where
        T: Fn(&FieldError<F>) -> S,
        S: Into<yew::Html>,
    {
        let mut translated = HashMap::<F, Vec<Rc<yew::Html>>>::new();
        for error in self.errors.iter() {
            let error_html = Rc::new(translator(error).into());
            for field in error.fields.iter() {
                translated
                    .entry(*field)
                    .and_modify(|errors| errors.push(error_html.clone()))
                    .or_insert_with(|| vec![error_html.clone()]);
            }
        }
        Rc::new(translated)
    }
}

impl<F> std::fmt::Display for ErrorSet<F>
where
    F: Copy + std::fmt::Debug,
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
        let mut errors = Vec::new();
        for (field, validation) in self.validations.into_iter() {
            if let Err(error) = validation.validate() {
                let mut fields = HashSet::new();
                fields.insert(field);
                errors.push(FieldError { fields, error });
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
    pub use super::{
        ErrorSet, FieldError, ModelValidator, Validatable, ValidationError, Validator,
    };
}
