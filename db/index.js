const { drizzle } = require("drizzle-orm/better-sqlite3");
// Imports Drizzle, the ORM we use to talk to the database.

const Database = require("better-sqlite3");
// Imports better-sqlite3, the actual database driver.

const sqlite = new Database("tickets.db");
// Creates (or opens) a file called tickets.db in the root folder.
// This is where all our data is physically stored.

const db = drizzle(sqlite);
// Wraps the database connection in Drizzle so we can use its cleaner syntax.

module.exports = { db, sqlite };
// Exports both. We use 'db' for Drizzle queries (in app.js) 
// and 'sqlite' for raw SQL queries (in technicians.js).
