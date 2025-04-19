mod contact;
mod contact_repository;

use anyhow::Result;
use automerge::AutoCommit;
use contact::Contact;
use contact_repository::ContactRepository;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::path::Path;
use std::fs;

async fn create_db_pool(db_path: &str) -> Result<SqlitePool> {
    // Remove the existing database if it exists
    if Path::new(db_path).exists() {
        fs::remove_file(db_path)?;
    }

    // Create the database
    if !Sqlite::database_exists(db_path).await? {
        Sqlite::create_database(db_path).await?
    }

    // Connect to the database
    let pool = SqlitePool::connect(db_path).await?;
    
    // Create the contacts table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS contacts (
            id TEXT PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            phone TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

async fn display_all_contacts(repo: &ContactRepository) -> Result<()> {
    println!("\n--- Current Contacts in Database ---");
    let contacts = repo.all().await?;
    
    if contacts.is_empty() {
        println!("No contacts found.");
    } else {
        for contact in contacts {
            println!("{} {}: {} / {} (Created: {}, Updated: {})", 
                contact.first_name, 
                contact.last_name,
                contact.phone,
                contact.email,
                contact.created_at,
                contact.updated_at
            );
        }
    }
    println!("----------------------------------\n");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the database
    let pool = create_db_pool("./contacts.sqlite").await?;
    let contact_repo = ContactRepository::new(pool);
    
    // Initialize Automerge document
    let mut doc = AutoCommit::new();

    println!("Step 1: Create a new contact and save it to Automerge");
    
    // Create a new contact
    let mut contact = Contact::new(
        "John", 
        "Doe", 
        "555-123-4567", 
        "john.doe@example.com"
    );
    
    // Save to Automerge document and get patches
    let before_heads = doc.get_heads();
    contact.save_to_automerge(&mut doc)?;
    let after_heads = doc.get_heads();
    let patches = doc.diff(&before_heads, &after_heads);
    
    println!("Created contact: {} {}", contact.first_name, contact.last_name);
    
    // Apply patches to the database
    println!("Step 2: Apply patches to the database");
    contact_repo.apply_patches(patches).await?;
    
    // Display contacts
    display_all_contacts(&contact_repo).await?;
    
    // Modify the contact
    println!("Step 3: Modify the contact in Automerge");
    contact.phone = "555-987-6543".to_string();
    contact.email = "john.d@company.com".to_string();
    contact.update(); // Update the updated_at timestamp
    
    // Save the updated contact to Automerge and get patches
    let before_heads = doc.get_heads();
    contact.save_to_automerge(&mut doc)?;
    let after_heads = doc.get_heads();
    let patches = doc.diff(&before_heads, &after_heads);
    
    // Apply patches to the database
    println!("Step 4: Apply updated patches to the database");
    contact_repo.apply_patches(patches).await?;
    
    // Display contacts after update
    display_all_contacts(&contact_repo).await?;
    
    // Delete the contact
    println!("Step 5: Delete the contact from Automerge");
    let before_heads = doc.get_heads();
    contact.delete_from_automerge(&mut doc)?;
    let after_heads = doc.get_heads();
    let patches = doc.diff(&before_heads, &after_heads);
    
    // Apply delete patches to the database
    println!("Step 6: Apply delete patches to the database");
    contact_repo.apply_patches(patches).await?;
    
    // Display contacts after deletion
    display_all_contacts(&contact_repo).await?;
    
    Ok(())
}