-- Billing addresses for local development users
INSERT INTO billjobs_userprofile (user_id, billing_address)
SELECT id, '1 rue de l''Admin, 75001 Paris' FROM auth_user WHERE username = 'admin'
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO billjobs_userprofile (user_id, billing_address)
SELECT id, '12 avenue des Lilas, 69003 Lyon' FROM auth_user WHERE username = 'alice'
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO billjobs_userprofile (user_id, billing_address)
SELECT id, '8 boulevard Victor Hugo, 06000 Nice' FROM auth_user WHERE username = 'bob'
ON CONFLICT (user_id) DO NOTHING;