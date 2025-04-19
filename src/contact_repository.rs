use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;
use automerge::Patch;
use automerge::PatchAction;

use crate::contact::Contact;

pub struct ContactRepository {
    pool: SqlitePool,
}

impl ContactRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Contact>> {
        let contact = sqlx::query_as::<_, Contact>(
            "SELECT id, first_name, last_name, phone, email, created_at, updated_at FROM contacts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(contact)
    }

    pub async fn insert(&self, contact: &Contact) -> Result<()> {
        sqlx::query(
            "INSERT INTO contacts (id, first_name, last_name, phone, email, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(contact.id)
        .bind(&contact.first_name)
        .bind(&contact.last_name)
        .bind(&contact.phone)
        .bind(&contact.email)
        .bind(contact.created_at)
        .bind(contact.updated_at)
        .execute(&self.pool)
        .await?;

        println!("Inserted contact: {} {}", contact.first_name, contact.last_name);
        Ok(())
    }

    pub async fn update(&self, contact: &Contact) -> Result<()> {
        sqlx::query(
            "UPDATE contacts 
             SET first_name = ?, last_name = ?, phone = ?, email = ?, updated_at = ? 
             WHERE id = ?",
        )
        .bind(&contact.first_name)
        .bind(&contact.last_name)
        .bind(&contact.phone)
        .bind(&contact.email)
        .bind(contact.updated_at)
        .bind(contact.id)
        .execute(&self.pool)
        .await?;

        println!("Updated contact: {} {}", contact.first_name, contact.last_name);
        Ok(())
    }
    
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM contacts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        println!("Deleted contact with ID: {}", id);
        Ok(())
    }

    pub async fn all(&self) -> Result<Vec<Contact>> {
        let contacts = sqlx::query_as::<_, Contact>(
            "SELECT id, first_name, last_name, phone, email, created_at, updated_at FROM contacts 
             ORDER BY last_name, first_name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(contacts)
    }

    pub async fn apply_patches(&self, patches: Vec<Patch>) -> Result<()> {
        if patches.is_empty() {
            return Ok(());
        }

        let patch_action = patches.first().unwrap().action.clone();
        
        match patch_action {
            PatchAction::PutMap {..} => {
                let contact = Contact::try_from(patches)?;

                if let Some(_existing_contact) = self.get_by_id(contact.id).await? {
                    self.update(&contact).await?;
                } else {
                    self.insert(&contact).await?;
                }
            },
            PatchAction::DeleteMap { key } => {
                let contact_id = Uuid::parse_str(&key)?;
                self.delete(contact_id).await?;
            },
            _ => {}
        };

        Ok(())
    }
}