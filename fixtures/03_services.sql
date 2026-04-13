-- Sample billjobs_service rows for local development

INSERT INTO
  billjobs_service (
    reference,
    "name",
    description,
    price,
    is_available
  )
VALUES
  (
    'PT002',
    'Part Time (15 jours /1 mois)',
    'Accès 15 jours par mois à l''espace',
    155.0,
    true
  ),
  (
    'MT001',
    'Mid Time (10 jours  / 1 mois)',
    'Un accès 10 jours par mois au coworking',
    115.0,
    true
  ),
  (
    'FT001',
    'Full Time (Accès libre / 1 mois)',
    'Accès à temps plein pendant 1 mois',
    195.0,
    true
  ),
  (
    'TD001',
    'Carnet (10 journées ou 20 demi-journées)',
    'Valable 1 an',
    140.0,
    true
  ),
  (
    'OD002',
    'Journée',
    'Accès une journée à l''espace de coworking',
    20.0,
    true
  ),
  (
    'OD005',
    'La demie (5h)',
    'Accès 5h à l''espace de coworking',
    12.0,
    true
  );