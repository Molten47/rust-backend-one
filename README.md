| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | `/auth/signup` | Register a new user | Public |
| POST | `/auth/login` | Login and receive JWT | Public |
| GET | `/auth/me` | Get current user profile | JWT Required |


## Validation Rules
- **Username** — 3–50 characters, letters/numbers/underscores only
- **Email** — must be a valid email format
- **Password** — minimum 8 characters, must contain uppercase, lowercase, and a number
