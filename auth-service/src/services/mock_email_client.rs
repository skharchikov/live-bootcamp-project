use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::{
    domain::{Email, EmailClient},
    EmailPayload,
};

#[derive(Debug, Default)]
pub struct MockEmailClient {
    pub sent: RwLock<HashMap<Email, EmailPayload>>,
}

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), String> {
        tracing::info!(
            "MockEmailClient: Sending email to {} with subject: {} and content: {}",
            recipient.as_ref(),
            subject,
            content
        );

        self.sent.write().await.insert(
            recipient.clone(),
            EmailPayload {
                recipient: recipient.clone(),
                subject: subject.to_string(),
                content: content.to_string(),
            },
        );

        Ok(())
    }
}
