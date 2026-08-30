const express = require("express");
const { db } = require("./db");
const { tickets, users, customers } = require("./db/schema");
const { desc, eq } = require("drizzle-orm");
const session = require("express-session");
const bcrypt = require("bcryptjs");

const app = express();
app.set("view engine", "ejs");
app.use(express.urlencoded({ extended: true }));
app.use(express.static("public"));
app.use(session({
  secret: "helpdesk-secret-key",
  resave: false,
  saveUninitialized: false,
}));

function requireLogin(req, res, next) {
  if (!req.session.userId) {
    return res.redirect("/login");
  }
  next();
}

function requireCustomer(req, res, next) {
  if (!req.session.customerId) {
    return res.redirect("/account/login");
  }
  next();
}

// ----- Staff auth -----

app.get("/login", (req, res) => {
  res.render("login");
});

app.post("/login", (req, res) => {
  const user = db.select().from(users).where(eq(users.username, req.body.username)).get();

  if (!user || !bcrypt.compareSync(req.body.password, user.password)) {
    return res.render("login", { error: "Invalid username or password" });
  }

  req.session.userId = user.id;
  res.redirect("/staff");
});

app.get("/logout", (req, res) => {
  req.session.destroy(() => {
    res.redirect("/login");
  });
});

// ----- Customer auth -----

app.get("/account/signup", (req, res) => {
  res.render("customer_signup");
});

app.post("/account/signup", (req, res) => {
  const existing = db.select().from(customers).where(eq(customers.email, req.body.email)).get();
  if (existing) {
    return res.render("customer_signup", { error: "An account with that email already exists." });
  }

  const hashedPassword = bcrypt.hashSync(req.body.password, 10);
  const result = db.insert(customers).values({
    name: req.body.name,
    email: req.body.email,
    password: hashedPassword,
  }).run();

  req.session.customerId = result.lastInsertRowid;
  res.redirect("/new/account");
});

app.get("/account/login", (req, res) => {
  res.render("customer_login");
});

app.post("/account/login", (req, res) => {
  const customer = db.select().from(customers).where(eq(customers.email, req.body.email)).get();

  if (!customer || !bcrypt.compareSync(req.body.password, customer.password)) {
    return res.render("customer_login", { error: "Invalid email or password" });
  }

  req.session.customerId = customer.id;
  res.redirect("/new/account");
});

app.get("/account/logout", (req, res) => {
  req.session.destroy(() => {
    res.redirect("/");
  });
});

// ----- Home -----

app.get("/", (req, res) => {
  const allTickets = db.select().from(tickets).all();
  const resolvedCount = allTickets.filter(t => t.status === "Resolved").length;
  const openCount = allTickets.filter(t => t.status !== "Resolved").length;
  const totalCount = allTickets.length;

  const resolvedPercent = totalCount > 0 ? (resolvedCount / totalCount) * 100 : 0;
  const openPercent = totalCount > 0 ? (openCount / totalCount) * 100 : 0;

  const circumference = 2 * Math.PI * 45;
  const resolvedArc = (resolvedPercent / 100) * circumference;
  const openArc = (openPercent / 100) * circumference;

  res.render("home", {
    totalCount,
    resolvedCount,
    openCount,
    circumference,
    resolvedArc,
    openArc,
  });
});

// ----- Ticket submission (choice + guest + account) -----

app.get("/new", (req, res) => {
  res.render("new_choice");
});

app.get("/new/guest", (req, res) => {
  res.render("new_guest");
});

app.post("/new/guest", (req, res) => {
  db.insert(tickets).values({
    subject: req.body.subject,
    description: req.body.description,
    priority: req.body.priority,
    category: req.body.category,
    submittedBy: req.body.name,
    customerId: null,
  }).run();
  res.redirect("/");
});

app.get("/new/account", requireCustomer, (req, res) => {
  const customer = db.select().from(customers).where(eq(customers.id, req.session.customerId)).get();
  res.render("new_account", { customer });
});

app.post("/new/account", requireCustomer, (req, res) => {
  const customer = db.select().from(customers).where(eq(customers.id, req.session.customerId)).get();

  db.insert(tickets).values({
    subject: req.body.subject,
    description: req.body.description,
    priority: req.body.priority,
    category: req.body.category,
    submittedBy: customer.name,
    customerId: customer.id,
  }).run();
  res.redirect("/");
});

// ----- Staff area -----

app.get("/staff", requireLogin, (req, res) => {
  const allTickets = db.select().from(tickets).orderBy(desc(tickets.openedAt)).all();
  res.render("tickets", { tickets: allTickets });
});

app.get("/ticket/:id", requireLogin, (req, res) => {
  const ticket = db.select().from(tickets).where(eq(tickets.id, Number(req.params.id))).get();

  let customerEmail = null;
  if (ticket.customerId) {
    const customer = db.select().from(customers).where(eq(customers.id, ticket.customerId)).get();
    customerEmail = customer ? customer.email : null;
  }

  res.render("ticket_detail", { ticket: ticket, customerEmail: customerEmail });
});

app.post("/ticket/:id", requireLogin, (req, res) => {
  const updates = { status: req.body.status };

  if (req.body.status === "Resolved") {
    updates.resolvedAt = new Date();
  } else {
    updates.resolvedAt = null;
  }

  db.update(tickets)
    .set(updates)
    .where(eq(tickets.id, Number(req.params.id)))
    .run();
  res.redirect("/ticket/" + req.params.id);
});

app.listen(3000, () => {
  console.log("Server running at http://localhost:3000");
});
