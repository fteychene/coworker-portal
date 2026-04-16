-- Add billline_id to portal_voucher so each voucher is associated with a specific
-- bill line rather than just a bill. This enables multi-line bill support.

ALTER TABLE portal_voucher ADD COLUMN billline_id INT;

-- Data migration: all existing portal_voucher rows were created by this portal,
-- which always wrote exactly one billjobs_billline per bill. Map each voucher to
-- its corresponding bill line via the shared bill_id.
UPDATE portal_voucher pv
SET billline_id = bl.id
FROM billjobs_billline bl
WHERE bl.bill_id = pv.bill_id;

-- Safety guard: abort if any voucher could not be matched.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM portal_voucher WHERE billline_id IS NULL) THEN
    RAISE EXCEPTION 'Data migration failed: some portal_voucher rows have no matching billjobs_billline';
  END IF;
END $$;

ALTER TABLE portal_voucher ALTER COLUMN billline_id SET NOT NULL;

ALTER TABLE portal_voucher DROP CONSTRAINT portal_voucher_pkey;