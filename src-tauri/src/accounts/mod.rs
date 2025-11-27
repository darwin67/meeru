use anyhow::{Context, Result};
use keyring::Entry;
use sqlx::SqlitePool;

use crate::db::models::{Account, AuthType};

// Test-only in-memory password storage (available in debug builds for testing)
#[cfg(debug_assertions)]
use std::collections::HashMap;
#[cfg(debug_assertions)]
use std::sync::Mutex;
#[cfg(debug_assertions)]
lazy_static::lazy_static! {
    static ref TEST_PASSWORDS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// Account manager handles account CRUD and credential storage
pub struct AccountManager {
    pool: SqlitePool,
}

impl AccountManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new email account
    pub async fn create_account(
        &self,
        email: String,
        name: Option<String>,
        provider: String,
        imap_host: String,
        imap_port: u16,
        smtp_host: String,
        smtp_port: u16,
        auth_type: AuthType,
        password: Option<String>,
    ) -> Result<Account> {
        let account = Account::new(
            email.clone(),
            name,
            provider,
            imap_host,
            imap_port,
            smtp_host,
            smtp_port,
            auth_type,
        );

        // Insert into database
        sqlx::query(
            r#"
            INSERT INTO accounts (id, email, name, provider, imap_host, imap_port, smtp_host, smtp_port, auth_type, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&account.id)
        .bind(&account.email)
        .bind(&account.name)
        .bind(&account.provider)
        .bind(&account.imap_host)
        .bind(account.imap_port)
        .bind(&account.smtp_host)
        .bind(account.smtp_port)
        .bind(&account.auth_type)
        .bind(account.created_at)
        .bind(account.updated_at)
        .execute(&self.pool)
        .await
        .context("Failed to insert account")?;

        // Store password in keychain if provided
        if let Some(pwd) = password {
            self.store_password(&account.id, &pwd)
                .context("Failed to store password in keychain")?;
        }

        Ok(account)
    }

    /// Get account by ID
    pub async fn get_account(&self, account_id: &str) -> Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT * FROM accounts WHERE id = ?
            "#,
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch account")?;

        Ok(account)
    }

    /// Get account by email
    pub async fn get_account_by_email(&self, email: &str) -> Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT * FROM accounts WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch account by email")?;

        Ok(account)
    }

    /// List all accounts
    pub async fn list_accounts(&self) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            r#"
            SELECT * FROM accounts ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list accounts")?;

        Ok(accounts)
    }

    /// Update account
    pub async fn update_account(&self, account: &Account) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE accounts
            SET email = ?, name = ?, provider = ?, imap_host = ?, imap_port = ?,
                smtp_host = ?, smtp_port = ?, auth_type = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&account.email)
        .bind(&account.name)
        .bind(&account.provider)
        .bind(&account.imap_host)
        .bind(account.imap_port)
        .bind(&account.smtp_host)
        .bind(account.smtp_port)
        .bind(&account.auth_type)
        .bind(chrono::Utc::now())
        .bind(&account.id)
        .execute(&self.pool)
        .await
        .context("Failed to update account")?;

        Ok(())
    }

    /// Delete account
    pub async fn delete_account(&self, account_id: &str) -> Result<()> {
        // Delete password from keychain
        let _ = self.delete_password(account_id); // Ignore errors if password doesn't exist

        // Delete from database (cascade will handle related data)
        sqlx::query(
            r#"
            DELETE FROM accounts WHERE id = ?
            "#,
        )
        .bind(account_id)
        .execute(&self.pool)
        .await
        .context("Failed to delete account")?;

        Ok(())
    }

    /// Update last sync time
    pub async fn update_last_sync(&self, account_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE accounts SET last_sync_at = ? WHERE id = ?
            "#,
        )
        .bind(chrono::Utc::now())
        .bind(account_id)
        .execute(&self.pool)
        .await
        .context("Failed to update last sync time")?;

        Ok(())
    }

    /// Store password in OS keychain (or in-memory for debug builds)
    fn store_password(&self, account_id: &str, password: &str) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            let mut passwords = TEST_PASSWORDS.lock().unwrap();
            passwords.insert(account_id.to_string(), password.to_string());
            return Ok(());
        }

        #[cfg(not(debug_assertions))]
        {
            let entry =
                Entry::new("meeru", account_id).context("Failed to create keyring entry")?;
            entry
                .set_password(password)
                .context("Failed to store password in keychain")?;
            Ok(())
        }
    }

    /// Retrieve password from OS keychain (or in-memory for debug builds)
    pub fn get_password(&self, account_id: &str) -> Result<String> {
        #[cfg(debug_assertions)]
        {
            let passwords = TEST_PASSWORDS.lock().unwrap();
            passwords
                .get(account_id)
                .cloned()
                .context("Password not found in test storage")
        }

        #[cfg(not(debug_assertions))]
        {
            let entry =
                Entry::new("meeru", account_id).context("Failed to create keyring entry")?;
            let password = entry
                .get_password()
                .context("Failed to retrieve password from keychain")?;
            Ok(password)
        }
    }

    /// Delete password from OS keychain (or in-memory for debug builds)
    fn delete_password(&self, account_id: &str) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            let mut passwords = TEST_PASSWORDS.lock().unwrap();
            passwords.remove(account_id);
            return Ok(());
        }

        #[cfg(not(debug_assertions))]
        {
            let entry =
                Entry::new("meeru", account_id).context("Failed to create keyring entry")?;
            entry
                .delete_credential()
                .context("Failed to delete password from keychain")?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use tempfile::TempDir;

    async fn create_test_account_manager() -> (AccountManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).await.unwrap();
        let manager = AccountManager::new(db.pool().clone());
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_create_and_get_account() {
        let (manager, _temp_dir) = create_test_account_manager().await;

        let account = manager
            .create_account(
                "test@example.com".to_string(),
                Some("Test User".to_string()),
                "gmail".to_string(),
                "imap.gmail.com".to_string(),
                993,
                "smtp.gmail.com".to_string(),
                587,
                AuthType::Password,
                Some("test_password".to_string()),
            )
            .await
            .unwrap();

        // Verify account was created
        let retrieved = manager.get_account(&account.id).await.unwrap().unwrap();
        assert_eq!(retrieved.email, "test@example.com");
        assert_eq!(retrieved.name, Some("Test User".to_string()));

        // Verify password is in keychain
        let password = manager.get_password(&account.id).unwrap();
        assert_eq!(password, "test_password");

        // Cleanup
        manager.delete_account(&account.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_list_accounts() {
        let (manager, _temp_dir) = create_test_account_manager().await;

        // Create multiple accounts
        let account1 = manager
            .create_account(
                "user1@example.com".to_string(),
                None,
                "gmail".to_string(),
                "imap.gmail.com".to_string(),
                993,
                "smtp.gmail.com".to_string(),
                587,
                AuthType::Password,
                None,
            )
            .await
            .unwrap();

        let account2 = manager
            .create_account(
                "user2@example.com".to_string(),
                None,
                "outlook".to_string(),
                "imap.outlook.com".to_string(),
                993,
                "smtp.outlook.com".to_string(),
                587,
                AuthType::OAuth2,
                None,
            )
            .await
            .unwrap();

        // List accounts
        let accounts = manager.list_accounts().await.unwrap();
        assert_eq!(accounts.len(), 2);

        // Cleanup
        manager.delete_account(&account1.id).await.unwrap();
        manager.delete_account(&account2.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_account() {
        let (manager, _temp_dir) = create_test_account_manager().await;

        let account = manager
            .create_account(
                "delete@example.com".to_string(),
                None,
                "gmail".to_string(),
                "imap.gmail.com".to_string(),
                993,
                "smtp.gmail.com".to_string(),
                587,
                AuthType::Password,
                Some("password".to_string()),
            )
            .await
            .unwrap();

        // Delete account
        manager.delete_account(&account.id).await.unwrap();

        // Verify account is deleted
        let retrieved = manager.get_account(&account.id).await.unwrap();
        assert!(retrieved.is_none());

        // Verify password is deleted from keychain
        let password_result = manager.get_password(&account.id);
        assert!(password_result.is_err());
    }
}
