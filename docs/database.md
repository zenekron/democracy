# Database

```mermaid
erDiagram
    INVITE_POLL {
        Uuid id PK
        GuildId guild_id
        UserId user_id
        DateTime created_at
        DateTime updated_at
    }

    INVITE_POLL_VOTE_SUBMISSION {
        Uuid invite_poll_id PK, FK
        UserId user_id PK
        InvitePollVote value
        DateTime created_at
        DateTime updated_at
    }

    INVITE_POLL ||--o{ INVITE_POLL_VOTE_SUBMISSION : "invite_poll_id"
```
