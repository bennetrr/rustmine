---
title: Multiplayer Architecture
sidebar:
  order: 2
---

## Security

There are three token types:

- ID token: Sent to the host to identify the guest
- Access token: Sent to the API to make authenticated requests
- Refresh token: Sent to the API to request a new set of tokens

The ID and access tokens have a short lifetime to limit the time in which stolen tokens can be used by unauthorized
actors. The refresh token has a longer lifetime so the player does not need to log in all the time.

The distinction between ID and access tokens is to prevent multiplayer hosts from making authorized
API requests as one of their guests (CWE-668).

## Host-to-Guest Communication

### Joining

```mermaid
sequenceDiagram
    participant API as Authentication Service
    participant G as Guest
    participant H as Host
    participant AG as All Guests
    G ->> API: Request ID token
    API -->> G: ID token
    G ->> H: Open WebSocket connection
    G ->> H: Authentication message (ID token)

    alt Player blocked by access list
        H -->> G: Deny access, close connection
    else Player allowed by access list
        H -->> G: Initial world data (player names, positions, chunks)
        H ->> AG: Announce new guest
    end
```

### World Updates (Example: Breaking Blocks)

```mermaid
sequenceDiagram
    participant G as Guest
    participant H as Host
    participant AG as All Guests

    alt Update initiated by a guest
        G ->> G: Update world (e.g. break block)
        G ->> H: Send world update
        H ->> H: Validate update and apply to world
    else Update initiated by the host
        H ->> H: Apply world update directly
    end
    H ->> AG: Broadcast world update
```

### Movement

```mermaid
sequenceDiagram
    participant G as Guest
    participant H as Host
    participant AG as All Guests

    alt Guest moves
        G ->> G: Move around
        G ->> H: Send movement update (optionally request new chunks)
        H ->> H: Validate movement and update player
    else Host moves
        H ->> H: Update player position directly
    end

    H ->> AG: Broadcast player update

    opt Chunks were requested
        H -->> G: Send requested chunks
    end
```
