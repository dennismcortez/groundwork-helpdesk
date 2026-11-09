# Groundwork: IT Help Desk Ticketing System

A full-stack IT help desk ticketing application designed to streamline internal IT support. Built with Node.js, Express, SQLite, and Drizzle ORM.

## Features

- **Automatic Ticket Assignment:** A custom SQL routing algorithm that assigns incoming tickets to the least-loaded technician based on their specific skills (e.g., Hardware, Network, Software).
- **Role-Based Authentication:** Separate login flows and protected routes for IT Staff and Customers using `express-session` and `bcrypt` password hashing.
- **SLA Tracking:** Color-coded Service Level Agreement indicators that track ticket deadlines based on priority (High: 4h, Medium: 24h, Low: 72h).
- **Reporting Dashboard:** A management analytics view featuring live Chart.js graphs for ticket volume, category breakdowns, and SLA compliance metrics.
- **Customer Portal:** Dedicated account history view allowing users to track the status of their previously submitted tickets.
- **Guest Submission:** Support for unauthenticated users to handle lockout scenarios.

## Tech Stack

- **Backend:** Node.js, Express.js
- **Frontend:** EJS (Embedded JavaScript templates), HTML/CSS
- **Database:** SQLite
- **ORM:** Drizzle ORM
- **Security:** BcryptJS, Express Session
- **Data Visualization:** Chart.js

## Database Architecture

The application utilizes a relational SQLite database with five tables:
1. `tickets` - Stores all submissions, statuses, and timestamps.
2. `users` - IT staff accounts.
3. `customers` - End-user accounts.
4. `technicians` - IT support agents.
5. `technician_categories` - A many-to-many join table linking technicians to the specific issue categories they are trained to handle.

## Local Setup & Installation

1. Clone the repository:
   git clone https://github.com/dennismcortez/groundwork-helpdesk.git
   cd groundwork-helpdesk

2. Install dependencies:
   npm install

3. Initialize the database and seed demo data:
   node reset_demo.js

4. Start the application:
   node app.js
   
The application will be running at http://localhost:3000.

## Demo Credentials

**Staff Login:** /login
- Username: dennis
- Password: helpdesk
