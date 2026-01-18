### `PORT`

Defines the TCP port on which the server listens.
Defaults to 2607 is not specified.

### `DATABASE_URL`

PostgreSQL connection string used by the backend.

### `JWT_SECRET`

Secret key used to sign and verify JSON Web Tokens.
This value must be kept private and secure.

---

## API Documentation

### Base URL

All API endpoints are prefixed with `/api`.

### Authentication

Most endpoints require authentication via JWT tokens passed in cookies. The JWT cookie is automatically set on successful login or registration.

Protected routes require the `Authorization` cookie containing a valid JWT token.

### Rate Limiting

- **Rate Limit**: 1 request per second per IP+route combination
- **Burst Size**: 20 requests

---

## REST API Endpoints

### Health Check

#### `GET /api/health`

Check if the server is running.

**Authentication**: None required

**Response**: `200 OK`
```
ok :)
```

---

### Authentication Endpoints

#### `POST /api/auth/register`

Register a new user account.

**Authentication**: None required

**Request Body**:
```json
{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePass123",
  "bio": "Optional bio text" // This is optional
}
```

**Validation Rules**:
- Email must be valid format
- Password must be at least 6 characters
- Password must contain at least one uppercase letter, one lowercase letter, and one digit

**Response**: `201 CREATED`
```json
{
  "ok": true,
  "message": "User registered successfully",
  "id": 123
}
```

**Error Responses**:
- `401 UNAUTHORIZED` - Validation failed
- `409 CONFLICT` - Username or email already exists
- `500 INTERNAL SERVER ERROR` - Database error

**Notes**: 
- Sets authentication cookie on success
- Password is hashed using Argon2 before storage

---

#### `POST /api/auth/login`

Authenticate an existing user.

**Authentication**: None required

**Request Body**:
```json
{
  "person": "john_doe",
  "password": "SecurePass123",
  "isEmail": false
}
```

**Parameters**:
- `person`: Username or email address
- `password`: User's password
- `isEmail`: Boolean indicating if `person` is an email address, if true you provide an email address or else you provide username.

**Response**: `200 OK`
```json
{
  "ok": true,
  "message": "Login successful",
  "id": 123
}
```

**Error Responses**:
- `400 BAD REQUEST` - Validation failed (empty fields or invalid email format)
- `401 UNAUTHORIZED` - Invalid credentials
- `500 INTERNAL SERVER ERROR` - Database error

**Notes**: 
- Sets authentication cookie on success
- Cookie expires after configured duration

---

### User Endpoints

#### `GET /api/users`

Get the authenticated user's profile information.

**Authentication**: Required (JWT cookie)

**Response**: `200 OK`
```json
{
  "email": "john@example.com",
  "username": "john_doe",
  "bio": "User bio text",
  "created_at": "2026-01-18T10:30:00Z",
  "updated_at": "2026-01-18T10:30:00Z"
}
```

**Error Responses**:
- `401 UNAUTHORIZED` - Invalid or missing JWT token
- `500 INTERNAL SERVER ERROR` - Database error

---

#### `PATCH /api/users`

Update the authenticated user's profile.

**Authentication**: Required (JWT cookie)

**Request Body**:
```json
{
  "email": "newemail@example.com",
  "username": "new_username",
  "bio": "Updated bio",
  "password": "CurrentPassword123",
  "new_password": "NewPassword456"
}
```

**Parameters** (all optional except `password`):
- `email`: New email address
- `username`: New username
- `bio`: New bio text
- `password`: Current password (required for verification)
- `new_password`: New password (optional, if changing password)

**Response**: `200 OK`
```json
{
  "updatedFields": ["email", "bio"]
}
```

**Error Responses**:
- `400 BAD REQUEST` - Validation failed or incorrect current password
- `401 UNAUTHORIZED` - Invalid or missing JWT token
- `409 CONFLICT` - New username or email already exists
- `500 INTERNAL SERVER ERROR` - Database error

**Notes**: 
- At least one field must be updated
- Password verification is required for all updates
- Returns list of successfully updated fields

---

### Chat Endpoints

#### `POST /api/chats/codes`

Create a new chat code for others to connect with you.

**Authentication**: Required (JWT cookie)

**Request Body**: None

**Response**: `201 CREATED`
```json
{
  "message": "Chat code created successfully",
  "code": 12345
}
```

**Error Responses**:
- `400 BAD REQUEST` - User already has 5 chat codes (maximum limit)
- `401 UNAUTHORIZED` - Invalid or missing JWT token
- `500 INTERNAL SERVER ERROR` - Database error

**Notes**: 
- Each user can have a maximum of 5 active chat codes
- Chat codes are random 5-digit numbers
- Codes are deleted after being used to create a conversation

---

#### `DELETE /api/chats/codes`

Delete an existing chat code.

**Authentication**: Required (JWT cookie)

**Request Body**:
```json
{
  "code": 12345
}
```

**Response**: `200 OK`
```json
{
  "message": "Chat code deleted successfully",
  "conversation_id": null
}
```

**Error Responses**:
- `401 UNAUTHORIZED` - Invalid or missing JWT token
- `404 NOT FOUND` - Chat code not found or not owned by user
- `500 INTERNAL SERVER ERROR` - Database error

---

#### `POST /api/chats`

Submit a chat code to start a conversation with another user.

**Authentication**: Required (JWT cookie)

**Request Body**:
```json
{
  "code": 12345
}
```

**Response**: `201 CREATED`
```json
{
  "message": "Conversation created successfully",
  "conversation_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Error Responses**:
- `400 BAD REQUEST` - Attempting to start conversation with yourself
- `401 UNAUTHORIZED` - Invalid or missing JWT token
- `404 NOT FOUND` - Chat code doesn't exist
- `409 CONFLICT` - Conversation already exists between users
- `500 INTERNAL SERVER ERROR` - Database error

**Notes**: 
- Chat code is automatically deleted after successful conversation creation
- Cannot create duplicate conversations between the same users
- Returns existing conversation if attempting to create duplicate

---

#### `GET /api/chats/messages`

Retrieve messages from a conversation with cursor-based pagination.

**Authentication**: Required (JWT cookie)

**Query Parameters**:
- `conversationId` (required): UUID of the conversation
- `cursor` (optional): RFC3339 timestamp for pagination (returns messages before this time)
- `limit` (optional): Number of messages to return (default: 50, max: 100)

**Example**: `/api/chats/messages?conversationId=550e8400-e29b-41d4-a716-446655440000&limit=20`

**Response**: `200 OK`
```json
{
  "chats": [
    {
      "id": "650e8400-e29b-41d4-a716-446655440001",
      "content": "Hello there!",
      "userSent": "john_doe",
      "sentAt": "2026-01-18T10:30:00Z"
    }
  ],
  "nextCursor": "2026-01-18T10:29:00Z",
  "hasMore": true
}
```

**Error Responses**:
- `401 UNAUTHORIZED` - Invalid or missing JWT token, or not a participant in the conversation
- `500 INTERNAL SERVER ERROR` - Database error

---

#### `PATCH /api/chats/messages`

Update an existing message in a conversation.

**Authentication**: Required (JWT cookie)

**Request Body**:
```json
{
  "conversationId": "550e8400-e29b-41d4-a716-446655440000",
  "messageId": "650e8400-e29b-41d4-a716-446655440001",
  "content": "Updated message text"
}
```

**Response**: `200 OK`
```json
{
  "message": "Message updated successfully",
  "editedAt": "2026-01-18T11:00:00Z"
}
```

**Error Responses**:
- `401 UNAUTHORIZED` - Invalid or missing JWT token, not message author
- `404 NOT FOUND` - Message not found
- `500 INTERNAL SERVER ERROR` - Database error

**Notes**: 
- Only the message author can edit their messages
- Updates the `edited_at` timestamp

---

#### `DELETE /api/chats/messages`

Delete a message from a conversation.

**Authentication**: Required (JWT cookie)

**Request Body**:
```json
{
  "conversationId": "550e8400-e29b-41d4-a716-446655440000",
  "messageId": "650e8400-e29b-41d4-a716-446655440001"
}
```

**Response**: `200 OK`
```json
{
  "message": "Message deleted successfully"
}
```

**Error Responses**:
- `401 UNAUTHORIZED` - Invalid or missing JWT token, not message author
- `404 NOT FOUND` - Message not found
- `500 INTERNAL SERVER ERROR` - Database error

**Notes**: 
- Only the message author can delete their messages

---

## WebSocket API

### Connection Endpoint

#### `WS /api/chats/ws`

Establish a WebSocket connection for real-time chat messaging.

**Authentication**: Required (JWT cookie)

**Query Parameters**:
- `chatId` (required): UUID of the conversation to connect to

**Example**: `ws://localhost:2607/api/chats/ws?chatId=550e8400-e29b-41d4-a716-446655440000`

**Connection Validation**:
- Verifies that the user is a participant in the specified conversation
- Rejects connection if user is not authorized

**Error Responses**:
- `400 BAD REQUEST` - Chat ID not provided
- `401 UNAUTHORIZED` - Invalid JWT token or not a participant in the conversation
- `500 INTERNAL SERVER ERROR` - Database error

---

### WebSocket Message Flow

#### Client to Server

Send messages by transmitting plain text through the WebSocket connection.

**Message Format**: Plain text string

**Example**:
```
Hello, how are you?
```

**Behavior**:
- Empty messages (only whitespace) are ignored
- Messages are persisted to the database immediately
- Messages are broadcast to other participants via PostgreSQL LISTEN/NOTIFY

---

#### Server to Client

Receive messages from other participants in real-time.

**Message Format**: Plain text string

**Example**:
```
I'm doing great, thanks for asking!
```

**Behavior**:
- Messages are delivered in real-time as they are sent by other participants
- You will NOT receive your own messages echoed back
- Connection uses PostgreSQL LISTEN/NOTIFY for efficient real-time updates
- Each conversation has its own notification channel: `conversation_{conversation_id}`

---

### WebSocket Event Types

#### Connection Lifecycle

1. **Connection Established**
   - Client successfully connects to the WebSocket
   - Server validates JWT and conversation participation
   - PostgreSQL listener is set up for the conversation channel

2. **Message Received (from client)**
   - Client sends text message
   - Server validates and persists to database
   - Database trigger sends notification to all connected clients
   
3. **Message Broadcast (to client)**
   - Server receives notification from PostgreSQL
   - Parses notification payload containing `user_id` and `content`
   - Broadcasts to all connected participants except the sender

4. **Connection Closed**
   - Client closes connection or encounters error
   - Server cleans up PostgreSQL listener
   - WebSocket connection terminates

#### Close Events

- Client sends `Close` frame
- Network error or timeout
- Server error (database failure, etc.)

---

### Implementation Details

**PostgreSQL Integration**:
- Uses PostgreSQL LISTEN/NOTIFY for real-time message broadcasting
- Each conversation has a dedicated channel: `conversation_{uuid}`
- Database trigger automatically sends notifications when messages are inserted

**Notification Payload Format** (internal):
```json
{
  "user_id": 123,
  "content": "Message text"
}
```

**Concurrency**:
- Uses Tokio's `select!` macro to handle concurrent WebSocket and database events
- Non-blocking message handling
- Automatic cleanup on connection drop

---

## Error Response Format

All error responses follow a consistent format:

```json
{
  "error": "Error message description"
}
```

Common HTTP status codes:
- `400 BAD REQUEST` - Invalid request data or validation failure
- `401 UNAUTHORIZED` - Authentication required or invalid credentials
- `404 NOT FOUND` - Resource not found
- `409 CONFLICT` - Resource conflict (e.g., duplicate username)
- `500 INTERNAL SERVER ERROR` - Server-side error

---

## Data Models

### User
```rust
{
  id: i64,              // Unique user ID
  username: String,     // Unique username
  email: String,        // Unique email address
  password_hash: String,// Argon2 hashed password
  bio: Option<String>,  // User biography
  created_at: DateTime, // Account creation timestamp
  updated_at: DateTime  // Last update timestamp
}
```

### Chat Code
```rust
{
  id: i64,       // Unique code ID
  code: u16,     // 5-digit numeric code
  user_id: i64,  // Owner's user ID
  created_at: DateTime
}
```

### Conversation
```rust
{
  id: Uuid,       // Unique conversation ID
  user_id_1: i64, // First participant (lower ID)
  user_id_2: i64, // Second participant (higher ID)
  created_at: DateTime
}
```

### Message
```rust
{
  id: Uuid,              // Unique message ID
  conversation_id: Uuid, // Parent conversation
  user_sent_id: i64,     // Sender's user ID
  content: String,       // Message content
  sent_at: DateTime,     // Send timestamp
  edited_at: Option<DateTime> // Last edit timestamp
}
```

---

## Security Notes

1. **Password Storage**: All passwords are hashed using Argon2 before storage
2. **JWT Tokens**: Signed with `JWT_SECRET` environment variable
3. **Cookie Security**: 
   - HttpOnly flag set
   - Secure flag set in production
   - SameSite policy applied
4. **Rate Limiting**: Per-IP + per-route rate limiting to prevent abuse
5. **SQL Injection**: All queries use parameterized statements via SQLx
6. **Input Validation**: All user inputs are validated before processing

---

## Database Schema

The application uses PostgreSQL with the following key tables:
- `users` - User accounts and authentication
- `chat_codes` - Temporary codes for initiating conversations
- `conversations` - Chat conversations between users
- `messages` - Individual chat messages
- `subscriptions` - Notification subscriptions (future use)

For detailed schema, see the migration files in the `migrations/` directory.

