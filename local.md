## Auth

### Login

```bash
# Login — success
curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "alicepass123"}' | jq .

# Expected response:
# { "token": "eyJ0eXAiOiJKV1QiLCJhbGci..." }

# Login — wrong credentials (returns 401)
curl -i -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "wrong"}'

# Using the token on a protected route
curl -s http://localhost:3000/api/some-protected-route \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGci..."
```
