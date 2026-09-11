const { db, sqlite } = require("./index");
const { technicians, technicianCategories } = require("./schema");

// Clear existing (safe to re-run)
sqlite.prepare("DELETE FROM technician_categories").run();
sqlite.prepare("DELETE FROM technicians").run();

const seed = [
  { name: "Alex Rivera",   email: "alex@groundwork.local",   skills: ["Hardware", "Network"] },
  { name: "Priya Shah",    email: "priya@groundwork.local",  skills: ["Software", "Account access"] },
  { name: "Marcus Chen",   email: "marcus@groundwork.local", skills: ["Network", "Software", "Hardware"] },
  { name: "Sofia Ortiz",   email: "sofia@groundwork.local",  skills: ["Account access", "Software"] },
];

for (const t of seed) {
  const result = db.insert(technicians).values({
    name: t.name,
    email: t.email,
    active: 1,
  }).run();

  const techId = result.lastInsertRowid;
  for (const cat of t.skills) {
    db.insert(technicianCategories).values({
      technicianId: techId,
      category: cat,
    }).run();
  }
  console.log(`Added ${t.name} (skills: ${t.skills.join(", ")})`);
}

console.log("\nSeeded technicians:");
const all = db.select().from(technicians).all();
for (const t of all) {
  const skills = sqlite.prepare("SELECT category FROM technician_categories WHERE technician_id = ?").all(t.id);
  console.log(`  #${t.id} ${t.name}: ${skills.map(s => s.category).join(", ")}`);
}
