INSERT INTO
    billjobs_bill (
        user_id,
        "number",
        "isPaid",
        billing_date,
        amount,
        issuer_address,
        billing_address
    )
VALUES
    (
        1,
        'F201801971',
        true,
        '2018-01-29',
        460.0,
        '
        Cowork''in Montpellier<br />
        ',
        'Admin billing'
    ),
    (
        2,
        'F201801973',
        true,
        '2018-01-31',
        25.5,
        '
        Cowork''in Montpellier<br />
        ',
        'Alice billing'
    ),
    (
        3,
        'F2018031013',
        true,
        '2018-03-20',
        190.0,
        '
        Cowork''in Montpellier<br />
        ',
        'Bob billing'
    );

INSERT INTO
    billjobs_billline (bill_id, service_id, quantity, total, note)
VALUES
    (1, 7, 1, 230.0, 'Decembre 2017'),
    (1, 7, 1, 230.0, 'Janvier 2018'),
    (2, 7, 2, 13.0, ''),
    (2, 7, 1, 12.5, ''),
    (3, 7, 1, 110.0, ''),
    (3, 7, 2, 80.0, '');