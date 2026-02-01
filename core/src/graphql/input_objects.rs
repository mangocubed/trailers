use async_graphql::InputObject;
use chrono::NaiveDate;

use crate::graphql::input_validators::*;

#[derive(InputObject)]
pub struct UserInputObject {
    #[graphql(validator(custom = UsernameValidator))]
    pub username: String,
    #[graphql(validator(custom = EmailValidator))]
    pub email: String,
    #[graphql(secret, validator(custom = PasswordValidator))]
    pub password: String,
    #[graphql(validator(custom  = FullNameValidator))]
    pub full_name: String,
    #[graphql(validator(custom  = BirthdateValidator))]
    pub birthdate: NaiveDate,
    #[graphql(validator(custom = CountryCodeValidator))]
    pub country_code: String,
}

#[derive(InputObject)]
pub struct SessionInputObject {
    #[graphql(validator(custom = UsernameOrEmailValidator))]
    pub username_or_email: String,
    #[graphql(secret, validator(custom = PasswordValidator))]
    pub password: String,
}
