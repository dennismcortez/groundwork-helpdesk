module.exports = {
  schema: "./db/schema.js",
  out: "./drizzle",
  dialect: "sqlite",
  dbCredentials: {
    url: "./tickets.db",
  },
};