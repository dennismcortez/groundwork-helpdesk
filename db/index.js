const { drizzle } = require("drizzle-orm/better-sqlite3");
const Database = require("better-sqlite3");

const sqlite = new Database("tickets.db");
const db = drizzle(sqlite);

module.exports = { db };