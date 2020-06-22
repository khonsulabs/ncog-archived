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
        match self.left.validate() {
            Ok(_) => self.right.validate(),
            Err(err) => Err(err),
        }
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
        self.left.validate().or_else(|_| self.right.validate())
    }
}

#[cfg(test)]
mod tests {
    use crate::validations::prelude::*;

    #[test]
    fn test_combinators() {
        let present_value = 1;
        let absent_value = 0;

        present_value
            .is_present()
            .or(absent_value.is_present())
            .validate()
            .expect("true or false = true");

        present_value
            .is_present()
            .or(present_value.is_present())
            .validate()
            .expect("true or true = true");

        absent_value
            .is_present()
            .or(present_value.is_present())
            .validate()
            .expect("false or true = true");

        absent_value
            .is_present()
            .or(absent_value.is_present())
            .validate()
            .expect_err("false or false = false");

        present_value
            .is_present()
            .and(absent_value.is_present())
            .validate()
            .expect_err("true and false = false");

        present_value
            .is_present()
            .and(present_value.is_present())
            .validate()
            .expect("true and true = true");

        absent_value
            .is_present()
            .and(present_value.is_present())
            .validate()
            .expect_err("false and true = false");

        absent_value
            .is_present()
            .and(absent_value.is_present())
            .validate()
            .expect_err("false and false = false");
    }
}
