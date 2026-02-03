use async_graphql::InputObject;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::graphql::input_validators::*;

#[derive(InputObject)]
pub struct SessionInputObject {
    #[graphql(validator(custom = UsernameOrEmailValidator))]
    pub username_or_email: String,
    #[graphql(secret, validator(custom = PasswordValidator))]
    pub password: String,
}

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
pub struct UserTitleTieInputObject {
    pub title_id: Uuid,
    pub is_checked: bool,
    pub video_id: Option<Uuid>,
}
