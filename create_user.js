const { db } = require("./db");
const { users } = require("./db/schema");
const bcrypt = require("bcryptjs");

const username = process.argv[2];
const password = process.argv[3];

if (!username || !password) {
  console.log("Usage: node create_user.js <username> <password>");
  process.exit(1);
}

const hashedPassword = bcrypt.hashSync(password, 10);

db.insert(users).values({
  username: username,
  password: hashedPassword,
}).run();

console.log("User created: " + username);