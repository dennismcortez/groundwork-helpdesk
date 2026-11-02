const Database = require('better-sqlite3');
const db = new Database('tickets.db');

// Clear existing tickets
db.prepare("DELETE FROM tickets").run();
// Reset the auto-increment counter back to 1
db.prepare("DELETE FROM sqlite_sequence WHERE name='tickets'").run();

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

const insert = db.prepare(`
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

  insert.run(t.subject, t.desc, t.priority, t.category, t.status, t.assigned, t.sub, openedAt, resolvedAt);
}

console.log("Database reset! Ready for practice.");
