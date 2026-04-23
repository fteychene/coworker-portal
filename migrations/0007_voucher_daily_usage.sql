ALTER TABLE portal_voucher
    ADD COLUMN active_days DATE[] NOT NULL DEFAULT '{}';
