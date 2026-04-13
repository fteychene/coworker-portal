# Coworking Tooling

## Fixtures

SQL files under `fixtures/` are loaded automatically when the Postgres container starts for the first time:

```bash
docker compose up -d
```

Credentials:

| Username | Password |
|----------|----------|
| admin | adminpass123 |
| alice | alicepass123 |
| bob | bobpass123 |

To regenerate `fixtures.sql` after editing `examples/gen_fixtures.rs`:

```bash
cargo run --example gen_fixtures > fixtures.sql
```
