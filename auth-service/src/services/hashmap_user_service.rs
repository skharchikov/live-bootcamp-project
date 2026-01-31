use std::collections::HashMap;

use crate::{domain::User, UserStore, UserStoreError};

#[derive(Debug, Default)]
pub struct HashmapUserStore {
    pub users: HashMap<String, User>,
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

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        self.users
            .get(email)
            .ok_or(UserStoreError::UserNotFound)
            .and_then(|user| {
                if user.password == password {
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
                "test@gmail.com".to_owned(),
                "12345!".to_owned(),
                false,
            ))
            .await;
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("test@gmail.com".to_owned(), "12345!".to_owned(), false);

        let insert_result = store.add_user(user.clone()).await;
        assert_eq!(insert_result, Ok(()));
        println!("Map state: {:?}", store.users);

        let result = store
            .get_user("test@gmail.com")
            .await
            .expect("User should be found");

        assert_eq!(result, user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("test@gmail.com".to_owned(), "12345!".to_owned(), false);

        let insert_result = store.add_user(user.clone()).await;
        assert_eq!(insert_result, Ok(()));

        let result = store
            .validate_user(user.email.as_ref(), user.password.as_ref())
            .await;

        assert_eq!(result, Ok(()));
    }
}
