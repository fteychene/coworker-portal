CREATE TABLE voucher (
    unify_id          VARCHAR(50)  NOT NULL PRIMARY KEY, -- UUID from Unify
    bill_id           INT          NOT NULL,             -- references billjobs_bill.id (external)
    unify_create_time BIGINT       NOT NULL,             -- Unix timestamp from Unify create response
    code              VARCHAR(10)  NOT NULL,
    created_at        TIMESTAMPTZ  NOT NULL,
    duration          INT          NOT NULL,             -- hours
    status            VARCHAR(10)  NOT NULL DEFAULT 'Valid'
                      CHECK (status IN ('Valid', 'Used', 'Expired', 'Unknown'))
);
