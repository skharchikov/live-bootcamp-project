use std::collections::HashMap;

use crate::{domain::User, Email, Password, UserStore, UserStoreError};

#[derive(Debug, Default)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.get(&user.email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(user.email.clone(), user);
                Ok(())
            }
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        self.users
            .get(email)
            .ok_or(UserStoreError::UserNotFound)
            .and_then(|user| {
                if user.password.eq(password) {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let result = store
            .add_user(User::new(
                Email::parse("test@gmail.com").unwrap(),
                Password::parse("123AB45!").unwrap(),
                false,
            ))
            .await;
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new(
            Email::parse("test@gmail.com").unwrap(),
            Password::parse("12345!AB").unwrap(),
            false,
        );

        let insert_result = store.add_user(user.clone()).await;
        assert_eq!(insert_result, Ok(()));
        println!("Map state: {:?}", store.users);

        let result = store
            .get_user(&Email::parse("test@gmail.com").unwrap())
            .await
            .expect("User should be found");

        assert_eq!(result, user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new(
            Email::parse("test@gmail.com").unwrap(),
            Password::parse("12345!AB").unwrap(),
            false,
        );

        let insert_result = store.add_user(user.clone()).await;
        assert_eq!(insert_result, Ok(()));

        let result = store.validate_user(&user.email, &user.password).await;

        assert_eq!(result, Ok(()));
    }
}
