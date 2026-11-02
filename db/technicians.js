const { db, sqlite } = require("./index");
// Imports the database connection.
const { technicians, tickets } = require("./schema");
// Imports the tables we need to query.
const { eq } = require("drizzle-orm");
// Drizzle helper for "WHERE column = value".

function assignTicket(ticketId, category) {
  // This function runs immediately after a ticket is submitted.
  // It takes the new ticket's ID and its category (e.g., "Network").

  const chosen = sqlite.prepare(`
    SELECT t.id, t.name
    FROM technicians t
    JOIN technician_categories tc ON tc.technician_id = t.id
    WHERE tc.category = ?
      AND t.active = 1
    ORDER BY (
      SELECT COUNT(*) FROM tickets 
      WHERE assigned_to = t.id AND status != 'Resolved'
    ) ASC, t.id ASC
    LIMIT 1
  `).get(category);

  // THE QUERY EXPLAINED:
  // 1. JOIN: Connect technicians to their skill categories.
  // 2. WHERE: Filter to only techs who have the matching skill, and are active.
  // 3. ORDER BY (The Magic): Counts how many open tickets each tech has. 
  //    "ASC" means lowest count first. The "t.id ASC" breaks ties.
  // 4. LIMIT 1: Only grab the top result (the least busy tech).
  // 5. .get(category): Passes the ticket's category into the "?" safely.

  if (!chosen) {
    return null;
    // If nobody has that skill, return null (ticket stays unassigned).
  }

  db.update(tickets)
    .set({ assignedTo: chosen.id })
    .where(eq(tickets.id, ticketId))
    .run();
    // Updates the new ticket, setting its assignedTo column to the winning tech's ID.
    
  return chosen;
}

function getTechnicianName(id) {
  // Helper function to turn a tech ID (like 3) into a name (like "Marcus Chen").
  if (id === null || id === undefined) return null;
  
  const t = db.select().from(technicians).where(eq(technicians.id, id)).get();
  return t ? t.name : null;
}

module.exports = { assignTicket, getTechnicianName };
