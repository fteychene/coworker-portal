INSERT INTO
    billjobs_bill (
        id, 
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
        947,
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
        949,
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
        990,
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
    (947, 7, 1, 230.0, 'Decembre 2017'),
    (947, 7, 1, 230.0, 'Janvier 2018'),
    (949, 7, 2, 13.0, ''),
    (949, 7, 1, 12.5, ''),
    (990, 7, 1, 110.0, ''),
    (990, 7, 2, 80.0, '');