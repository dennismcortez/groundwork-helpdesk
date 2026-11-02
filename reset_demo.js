const Database = require('better-sqlite3');
const fs = require('fs');
const bcrypt = require('bcryptjs');

// 1. Delete the old database file
if (fs.existsSync('tickets.db')) {
  fs.unlinkSync('tickets.db');
}

// 2. Create a brand new database
const db = new Database('tickets.db');

// 3. Recreate the tables
db.exec(`
CREATE TABLE tickets (
  id INTEGER PRIMARY KEY AUTOINCREMENT, subject TEXT NOT NULL, description TEXT NOT NULL,
  priority TEXT NOT NULL DEFAULT 'Medium', category TEXT NOT NULL DEFAULT 'Software',
  status TEXT NOT NULL DEFAULT 'Open', assigned_to INTEGER, submitted_by TEXT,
  customer_id INTEGER, opened_at INTEGER NOT NULL, resolved_at INTEGER
);
CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL UNIQUE, password TEXT NOT NULL
);
CREATE TABLE customers (
  id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, email TEXT NOT NULL UNIQUE, password TEXT NOT NULL
);
CREATE TABLE technicians (
  id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, email TEXT NOT NULL UNIQUE, active INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE technician_categories (
  technician_id INTEGER NOT NULL, category TEXT NOT NULL, PRIMARY KEY (technician_id, category)
);
`);

// 4. Create your admin user
const hashedPassword = bcrypt.hashSync("helpdesk", 10);
db.prepare("INSERT INTO users (username, password) VALUES (?, ?)").run("dennis", hashedPassword);

// 5. Recreate the Technicians
const techs = [
  { name: "Alex Rivera", email: "alex@groundwork.local", skills: ["Hardware", "Network"] },
  { name: "Priya Shah", email: "priya@groundwork.local", skills: ["Software", "Account access"] },
  { name: "Marcus Chen", email: "marcus@groundwork.local", skills: ["Network", "Software", "Hardware"] },
  { name: "Sofia Ortiz", email: "sofia@groundwork.local", skills: ["Account access", "Software"] }
];
const insertTech = db.prepare("INSERT INTO technicians (name, email, active) VALUES (?, ?, 1)");
const insertSkill = db.prepare("INSERT INTO technician_categories (technician_id, category) VALUES (?, ?)");
for (let t of techs) {
  const res = insertTech.run(t.name, t.email);
  for (let s of t.skills) insertSkill.run(res.lastInsertRowid, s);
}

// 6. Inject the 6 perfect Demo Tickets
const now = Math.floor(Date.now() / 1000);
const HOUR = 3600;
const tickets = [
  { subject: "Laptop screen flickering", desc: "The display goes black every few seconds.", priority: "High", category: "Hardware", status: "Open", assigned: 3, sub: "Sarah J." },
  { subject: "Install Adobe Illustrator", desc: "Need this for the new marketing campaign.", priority: "Low", category: "Software", status: "Open", assigned: 4, sub: "Mike T." },
  { subject: "Forgot email password", desc: "Locked out of my account, need a reset.", priority: "Medium", category: "Account access", status: "Resolved", assigned: 2, sub: "Elena R." },
  { subject: "VPN dropping connection", desc: "Cannot stay connected to the remote server.", priority: "Medium", category: "Network", status: "Open", assigned: 3, sub: "David L." },
  { subject: "Printer on 3rd floor jammed", desc: "Paper jam error that won't clear.", priority: "High", category: "Hardware", status: "Resolved", assigned: 3, sub: "Jessica M." },
  { subject: "Office WiFi is very slow", desc: "Pages are taking forever to load.", priority: "High", category: "Network", status: "Open", assigned: 3, sub: "Tom B." }
];
const insertTicket = db.prepare(`
  INSERT INTO tickets (subject, description, priority, category, status, assigned_to, submitted_by, opened_at, resolved_at)
  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
`);
for (let t of tickets) {
  let hoursAgo = 2; 
  if (t.priority === "High" && t.status === "Open") hoursAgo = 5; 
  if (t.priority === "Medium" && t.status === "Open") hoursAgo = 20; 
  if (t.priority === "Low" && t.status === "Open") hoursAgo = 1; 
  const openedAt = now - (hoursAgo * HOUR);
  const resolvedAt = t.status === "Resolved" ? openedAt + HOUR : null;
  insertTicket.run(t.subject, t.desc, t.priority, t.category, t.status, t.assigned, t.sub, openedAt, resolvedAt);
}

console.log("SUCCESS: Database wiped, tables rebuilt, and 6 demo tickets injected!");
