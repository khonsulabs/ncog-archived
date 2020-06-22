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
    #[error("should be blank")]
    NotAbsent,
    /// For when converting from a string to another type fails. Should be validated in another way.
    #[error("invalid value")]
    InvalidValue,
    #[error("custom error: {0}")]
    Custom(&'static str),
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

impl<T> ValidatorCombinators for T where T: Validator {}

pub trait ValidatableStorage<T>
where
    T: Clone + Default + PartialEq + std::fmt::Debug,
{
    fn as_form_storage(&self) -> FormStorage<T>;
}

impl<T> ValidatableStorage<T> for T
where
    T: Clone + Default + PartialEq + std::fmt::Debug,
{
    fn as_form_storage(&self) -> FormStorage<T> {
        FormStorage::new(self.clone())
    }
}

impl<T> ValidatableStorage<T> for FormStorage<T>
where
    T: Clone + Default + PartialEq + std::fmt::Debug,
{
    fn as_form_storage(&self) -> FormStorage<T> {
        self.clone()
    }
}

pub trait Validatable<T>
where
    T: Clone + Default + PartialEq + std::fmt::Debug,
{
    fn is_present(&self) -> PresentValidation<T>;
    fn is_absent(&self) -> AbsentValidation<T>;
}

impl<T, S> Validatable<T> for S
where
    T: Presentable + Clone + std::fmt::Debug,
    S: ValidatableStorage<T>,
{
    fn is_present(&self) -> PresentValidation<T> {
        PresentValidation {
            value: self.as_form_storage(),
        }
    }
    fn is_absent(&self) -> AbsentValidation<T> {
        AbsentValidation {
            value: self.as_form_storage(),
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

struct FieldValidator<F>
where
    F: std::fmt::Debug + std::hash::Hash + std::cmp::Eq,
{
    error_message: Option<&'static str>,
    fields: HashSet<F>,
    validator: Box<dyn Validator>,
}

pub struct ModelValidator<F>
where
    F: std::fmt::Debug + std::hash::Hash + std::cmp::Eq,
{
    validations: Vec<FieldValidator<F>>,
}

impl<F> Default for ModelValidator<F>
where
    F: std::fmt::Debug + std::hash::Hash + std::cmp::Eq,
{
    fn default() -> Self {
        Self {
            validations: Vec::default(),
        }
    }
}

impl<F> ModelValidator<F>
where
    F: Copy + std::fmt::Debug + std::hash::Hash + std::cmp::Eq,
{
    pub fn with_field<V: Validator + 'static>(mut self, field: F, validator: V) -> Self {
        self.validations.push(FieldValidator {
            fields: vec![field].into_iter().collect(),
            validator: Box::new(validator),
            error_message: None,
        });
        self
    }
    pub fn with_custom<V: Validator + 'static>(
        mut self,
        field: F,
        validator: V,
        message: &'static str,
    ) -> Self {
        self.validations.push(FieldValidator {
            fields: vec![field].into_iter().collect(),
            validator: Box::new(validator),
            error_message: Some(message),
        });
        self
    }
    pub fn with_fields<V: Validator + 'static, I: std::iter::Iterator<Item = F>>(
        mut self,
        fields: I,
        validator: V,
        error_message: &'static str,
    ) -> Self {
        self.validations.push(FieldValidator {
            fields: fields.collect(),
            validator: Box::new(validator),
            error_message: Some(error_message),
        });
        self
    }
    pub fn validate(self) -> Option<Rc<ErrorSet<F>>> {
        let mut errors = Vec::new();
        for validation in self.validations.into_iter() {
            if let Err(error) = validation.validator.validate() {
                errors.push(FieldError {
                    fields: validation.fields,
                    error: validation
                        .error_message
                        .map(|m| ValidationError::Custom(m))
                        .unwrap_or(error),
                });
            }
        }

        if !errors.is_empty() {
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
        ValidatorCombinators,
    };
}
