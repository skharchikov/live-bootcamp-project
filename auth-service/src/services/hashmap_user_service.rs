use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Debug, Default)]
pub struct HashmapUserStore {
    pub users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.get(&user.email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(user.email.clone(), user);
                Ok(())
            }
        }
    }

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
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

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let result = store.add_user(User::new(
            "test@gmail.com".to_owned(),
            "12345!".to_owned(),
            false,
        ));
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("test@gmail.com".to_owned(), "12345!".to_owned(), false);

        let insert_result = store.add_user(user.clone());
        assert_eq!(insert_result, Ok(()));
        println!("Map state: {:?}", store.users);

        let result = store
            .get_user("test@gmail.com")
            .expect("User should be found");

        assert_eq!(result, user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("test@gmail.com".to_owned(), "12345!".to_owned(), false);

        let insert_result = store.add_user(user.clone());
        assert_eq!(insert_result, Ok(()));

        let result = store.validate_user(user.email.as_ref(), user.password.as_ref());

        assert_eq!(result, Ok(()));
    }
}
