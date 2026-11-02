const { sqliteTable, integer, text, primaryKey } = require("drizzle-orm/sqlite-core");
// Pulls in the tools needed to define tables, columns, and primary keys.

// --- TICKETS TABLE ---
const tickets = sqliteTable("tickets", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  // Unique ID for every ticket (1, 2, 3...).
  
  subject: text("subject").notNull(),
  description: text("description").notNull(),
  priority: text("priority").notNull().default("Medium"),
  category: text("category").notNull().default("Software"),
  status: text("status").notNull().default("Open"),
  
  assignedTo: integer("assigned_to"),
  // Foreign key linking to the technicians table. Null if unassigned.
  
  submittedBy: text("submitted_by"),
  // Stores the name of a guest. Null if submitted by an account.
  
  customerId: integer("customer_id"),
  // Foreign key linking to the customers table. Null if guest.
  
  openedAt: integer("opened_at", { mode: "timestamp" }).notNull().$defaultFn(() => new Date()),
  // Timestamp of when the ticket was created. Used for SLA math.
  
  resolvedAt: integer("resolved_at", { mode: "timestamp" }),
  // Timestamp of when it was resolved. Null if still open.
});

// --- USERS TABLE (Staff) ---
const users = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  username: text("username").notNull().unique(),
  // unique() means two staff members can't have the same username.
  password: text("password").notNull(),
  // Stores the bcrypt hash, NEVER the raw password.
});

// --- CUSTOMERS TABLE (End Users) ---
const customers = sqliteTable("customers", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: text("name").notNull(),
  email: text("email").notNull().unique(),
  password: text("password").notNull(),
});

// --- TECHNICIANS TABLE ---
const technicians = sqliteTable("technicians", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: text("name").notNull(),
  email: text("email").notNull().unique(),
  active: integer("active").notNull().default(1),
  // 1 means they are working, 0 means they are inactive. 
  // We only assign tickets to active techs.
});

// --- TECHNICIAN CATEGORIES (Join Table) ---
const technicianCategories = sqliteTable("technician_categories", {
  technicianId: integer("technician_id").notNull(),
  category: text("category").notNull(),
}, (table) => ({
  pk: primaryKey({ columns: [table.technicianId, table.category] }),
  // This sets a composite primary key. A tech can only be assigned 
  // to a specific category once.
}));

module.exports = { tickets, users, customers, technicians, technicianCategories };
// Exports all tables so app.js can query them.
