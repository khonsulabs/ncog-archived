use super::{ValidationError, Validator};

#[derive(Debug)]
pub struct AndValidation<T, U>
where
    T: std::fmt::Debug,
    U: std::fmt::Debug,
{
    pub left: T,
    pub right: U,
}

impl<T, U> Validator for AndValidation<T, U>
where
    T: Validator + std::fmt::Debug,
    U: Validator + std::fmt::Debug,
{
    fn validate(&self) -> Result<(), ValidationError> {
        self.left.validate().or_else(|_| self.right.validate())
    }
}

#[derive(Debug)]
pub struct OrValidation<T, U>
where
    T: std::fmt::Debug,
    U: std::fmt::Debug,
{
    pub left: T,
    pub right: U,
}

impl<T, U> Validator for OrValidation<T, U>
where
    T: Validator + std::fmt::Debug,
    U: Validator + std::fmt::Debug,
{
    fn validate(&self) -> Result<(), ValidationError> {
        match self.left.validate() {
            Ok(_) => self.right.validate(),
            Err(err) => Err(err),
        }
    }
}
