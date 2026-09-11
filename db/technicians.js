const { db, sqlite } = require("./index");
const { technicians, tickets } = require("./schema");
const { eq } = require("drizzle-orm");

/**
 * Assign a ticket to the best-fit technician.
 *
 * Rules:
 *   1. Only technicians whose skills include the ticket's category qualify.
 *   2. Only active technicians are considered.
 *   3. Among qualified technicians, the one with the fewest open tickets wins.
 *   4. Ties are broken by lowest technician ID (deterministic).
 */
function assignTicket(ticketId, category) {
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

  if (!chosen) {
    return null;
  }

  db.update(tickets)
    .set({ assignedTo: chosen.id })
    .where(eq(tickets.id, ticketId))
    .run();

  return chosen;
}

function getTechnicianName(id) {
  if (id === null || id === undefined) return null;
  const t = db.select().from(technicians).where(eq(technicians.id, id)).get();
  return t ? t.name : null;
}

module.exports = { assignTicket, getTechnicianName };
