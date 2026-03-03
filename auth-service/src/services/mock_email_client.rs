use crate::domain::{Email, EmailClient};

#[derive(Debug, Default)]
pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), String> {
        // Our mock email client will simply log the recipient, subject, and content to standard output
        tracing::info!(
            "MockEmailClient: Sending email to {} with subject: {} and content: {}",
            recipient.as_ref(),
            subject,
            content
        );

        Ok(())
    }
}
