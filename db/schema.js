const { sqliteTable, integer, text, primaryKey } = require("drizzle-orm/sqlite-core");

const tickets = sqliteTable("tickets", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  subject: text("subject").notNull(),
  description: text("description").notNull(),
  priority: text("priority").notNull().default("Medium"),
  category: text("category").notNull().default("Software"),
  status: text("status").notNull().default("Open"),
  assignedTo: integer("assigned_to"),
  submittedBy: text("submitted_by"),
  customerId: integer("customer_id"),
  openedAt: integer("opened_at", { mode: "timestamp" }).notNull().$defaultFn(() => new Date()),
  resolvedAt: integer("resolved_at", { mode: "timestamp" }),
});

const users = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  username: text("username").notNull().unique(),
  password: text("password").notNull(),
});

const customers = sqliteTable("customers", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: text("name").notNull(),
  email: text("email").notNull().unique(),
  password: text("password").notNull(),
});

const technicians = sqliteTable("technicians", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: text("name").notNull(),
  email: text("email").notNull().unique(),
  active: integer("active").notNull().default(1),
});

const technicianCategories = sqliteTable("technician_categories", {
  technicianId: integer("technician_id").notNull(),
  category: text("category").notNull(),
}, (table) => ({
  pk: primaryKey({ columns: [table.technicianId, table.category] }),
}));

module.exports = { tickets, users, customers, technicians, technicianCategories };
