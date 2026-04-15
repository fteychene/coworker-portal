-- Dedicated table for guest bill access tokens.
-- Avoids modifying billjobs_bill (owned by the external django-billjobs app).
CREATE TABLE IF NOT EXISTS portal_guest_bill (
    guest_token UUID PRIMARY KEY,
    bill_id     INTEGER NOT NULL
);
