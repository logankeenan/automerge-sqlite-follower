# Automerge Sqlite Follower

A concise example demonstrating how to synchronize an Automerge CRDT document with a SQLite database.

## Overview

This project shows a simple pattern for:

1. Making changes to data in an Automerge document
2. Extracting patches from those changes
3. Applying those patches to update a SQLite database

## Key Concepts

- **Automerge Document**: The source of truth for data changes
- **Patches**: Generated when the Automerge document changes
- **SQLite Database**: Updated by interpreting and applying patches

## Why This Pattern Matters

This pattern enables developers to get the best of both worlds:
- Automerge handles the complexity of data synchronization, conflict resolution, and change management
- SQLite provides a familiar, query-optimized database experience for application development
- Changes can propagate across peers while maintaining a standard database-backed application architecture
- Developers can leverage SQL for queries and local operations while still building collaborative, offline-capable apps

## Running the Example

```bash
cargo run
```

This will:
1. Create a new contact in the Automerge document
2. Apply the patches to insert into SQLite
3. Update the contact in Automerge
4. Apply patches to update SQLite
5. Delete the contact in Automerge
6. Apply patches to delete from SQLite

