use async_graphql::{CustomValidator, InputType, InputValueError, Value};
use chrono::{NaiveDate, Utc};
use regex::Regex;

use crate::block_on;
use crate::commands::{user_email_exists, user_username_exists};
use crate::constants::{REGEX_EMAIL, REGEX_USERNAME};

pub struct BirthdateValidator;

impl CustomValidator<NaiveDate> for BirthdateValidator {
    fn check(&self, value: &NaiveDate) -> Result<(), InputValueError<NaiveDate>> {
        if *value > Utc::now().date_naive() {
            Err(input_value_error("birthdate", "Is invalid"))
        } else {
            Ok(())
        }
    }
}

pub struct CountryCodeValidator;

impl CustomValidator<String> for CountryCodeValidator {
    fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
        validate_presence("countryCode", value)?;
        validate_length("countryCode", value, 2, 2)?;

        Ok(())
    }
}

pub struct EmailValidator;

impl CustomValidator<String> for EmailValidator {
    fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
        validate_presence("email", value)?;
        validate_length("email", value, 5, 255)?;
        validate_format("email", value, &REGEX_EMAIL)?;

        if block_on(user_email_exists(value)) {
            Err(input_value_error("email", "Already exists"))
        } else {
            Ok(())
        }
    }
}

pub struct FullNameValidator;

impl CustomValidator<String> for FullNameValidator {
    fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
        validate_length("fullName", value, 1, 255)?;

        Ok(())
    }
}

pub struct PasswordValidator;

impl CustomValidator<String> for PasswordValidator {
    fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
        validate_presence("password", value)?;
        validate_length("password", value, 6, 255)?;

        Ok(())
    }
}

pub struct UsernameOrEmailValidator;

impl CustomValidator<String> for UsernameOrEmailValidator {
    fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
        validate_presence("usernameOrEmail", value)?;
        validate_length("usernameOrEmail", value, 3, 255)?;
        validate_format("usernameOrEmail", value, &REGEX_USERNAME)
            .or_else(|_| validate_format("usernameOrEmail", value, &REGEX_EMAIL))?;

        Ok(())
    }
}

pub struct UsernameValidator;

impl CustomValidator<String> for UsernameValidator {
    fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
        validate_presence("username", value)?;
        validate_length("username", value, 3, 16)?;
        validate_format("username", value, &REGEX_USERNAME)?;

        if block_on(user_username_exists(value)) {
            Err(input_value_error("username", "Already exists"))
        } else {
            Ok(())
        }
    }
}

fn input_value_error<T: InputType>(field_name: &str, message: &str) -> InputValueError<T> {
    InputValueError::custom(message).with_extension(
        "inputErrors",
        Value::from_json(serde_json::json!({ field_name: message })).unwrap(),
    )
}

fn validate_format(field_name: &str, value: &str, regex: &Regex) -> Result<(), InputValueError<String>> {
    if !regex.is_match(value) {
        Err(input_value_error(field_name, "Is invalid"))
    } else {
        Ok(())
    }
}

fn validate_length(field_name: &str, value: &str, min: usize, max: usize) -> Result<(), InputValueError<String>> {
    let value_len = value.len();

    let message = if value_len < min {
        format!("Must be at least {} characters long", min)
    } else if value_len > max {
        format!("Must be at most {} characters long", max)
    } else {
        return Ok(());
    };

    Err(input_value_error(field_name, &message))
}

fn validate_presence(field_name: &str, value: &str) -> Result<(), InputValueError<String>> {
    if value.is_empty() {
        Err(input_value_error(field_name, "Can't be blank"))
    } else {
        Ok(())
    }
}
