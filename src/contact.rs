use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use automerge::{transaction::Transactable, AutoCommit, ObjType, Patch, PatchAction, ROOT};
use serde::Serialize;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Contact {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Contact {
    pub fn new(first_name: &str, last_name: &str, phone: &str, email: &str) -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Contact {
            id: Uuid::new_v4(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            phone: phone.to_string(),
            email: email.to_string(),
            created_at: current_time,
            updated_at: current_time,
        }
    }

    pub fn update(&mut self) {
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
    }

    pub fn save_to_automerge(&self, doc: &mut AutoCommit) -> Result<()> {
        let contact_obj = doc.put_object(ROOT, self.id.to_string(), ObjType::Map)?;

        doc.put(&contact_obj, "id", self.id.to_string())?;
        doc.put(&contact_obj, "first_name", self.first_name.clone())?;
        doc.put(&contact_obj, "last_name", self.last_name.clone())?;
        doc.put(&contact_obj, "phone", self.phone.clone())?;
        doc.put(&contact_obj, "email", self.email.clone())?;
        doc.put(&contact_obj, "created_at", self.created_at.to_string())?;
        doc.put(&contact_obj, "updated_at", self.updated_at.to_string())?;

        Ok(())
    }

    pub fn delete_from_automerge(&self, doc: &mut AutoCommit) -> Result<()> {
        doc.delete(&ROOT, self.id.to_string())?;
        Ok(())
    }
}

impl TryFrom<Vec<Patch>> for Contact {
    type Error = anyhow::Error;

    fn try_from(patches: Vec<Patch>) -> Result<Self, Self::Error> {
        let mut first_name = String::new();
        let mut last_name = String::new();
        let mut phone = String::new();
        let mut email = String::new();
        let mut created_at = 0i64;
        let mut updated_at = 0i64;

        // Extract contact_id from the first patch
        let contact_id = if let Some(first_patch) = patches.first() {
            if let PatchAction::PutMap { key, .. } = &first_patch.action {
                Uuid::parse_str(key)?
            } else {
                return Err(anyhow!("First patch is not PutMap"));
            }
        } else {
            return Err(anyhow!("No patches provided"));
        };

        // Extract other fields from patches
        for patch in patches {
            if let PatchAction::PutMap { key, value, .. } = patch.action {
                match key.as_str() {
                    "first_name" => {
                        if let (automerge::Value::Scalar(val), _) = value {
                            first_name = val.to_string();
                        }
                    }
                    "last_name" => {
                        if let (automerge::Value::Scalar(val), _) = value {
                            last_name = val.to_string();
                        }
                    }
                    "phone" => {
                        if let (automerge::Value::Scalar(val), _) = value {
                            phone = val.to_string();
                        }
                    }
                    "email" => {
                        if let (automerge::Value::Scalar(val), _) = value {
                            email = val.to_string();
                        }
                    }
                    "created_at" => {
                        if let (automerge::Value::Scalar(val), _) = value {
                            created_at = val.to_str()
                                .ok_or_else(|| anyhow!("Invalid created_at value"))?
                                .parse::<i64>()?;
                        }
                    }
                    "updated_at" => {
                        if let (automerge::Value::Scalar(val), _) = value {
                            updated_at = val.to_str()
                                .ok_or_else(|| anyhow!("Invalid updated_at value"))?
                                .parse::<i64>()?;
                        }
                    }
                    _ => {}
                }
            }
        }

        if first_name.is_empty() || last_name.is_empty() {
            return Err(anyhow!("Required fields missing in patches"));
        }

        Ok(Contact {
            id: contact_id,
            first_name,
            last_name,
            phone,
            email,
            created_at,
            updated_at,
        })
    }
}