use apalis::prelude::BoxDynError;

use toolbox::config::MAILER_CONFIG;
use toolbox::identity_client::IdentityClient;
use toolbox::mailer::send_email;

use trailers_core::models::User;

pub async fn send_welcome_email(identity_client: &IdentityClient, user: &User) -> Result<(), BoxDynError> {
    let identity_user = user.identity_user(identity_client).await?;

    let message = format!(
        "Hello @{},

        Welcome to Filmstrip.

        If you have any questions, please contact us at the following email address: {}",
        identity_user.username, MAILER_CONFIG.support_email_address
    );

    Ok(send_email(&identity_user.email, "Welcome to Filmstrip", &message).await?)
}

pub mod admin_emails {
    use super::IdentityClient;

    use super::*;

    pub async fn send_new_user_email(identity_client: &IdentityClient, user: &User) -> Result<(), BoxDynError> {
        let identity_user = user.identity_user(identity_client).await?;

        let message = format!(
            "Hello,

Someone has created a new user account with the following username: @{}",
            identity_user.username
        );

        Ok(send_email(
            &MAILER_CONFIG.support_email_address,
            "(Admin) New user account created",
            &message,
        )
        .await?)
    }
}
