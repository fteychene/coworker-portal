-- Django auth_user table
CREATE TABLE IF NOT EXISTS auth_user (
    id           SERIAL PRIMARY KEY,
    password     VARCHAR(128)              NOT NULL,
    last_login   TIMESTAMP WITH TIME ZONE,
    is_superuser BOOLEAN                   NOT NULL,
    username     VARCHAR(150)              NOT NULL UNIQUE,
    first_name   VARCHAR(150)              NOT NULL,
    last_name    VARCHAR(150)              NOT NULL,
    email        VARCHAR(254)              NOT NULL,
    is_staff     BOOLEAN                   NOT NULL,
    is_active    BOOLEAN                   NOT NULL,
    date_joined  TIMESTAMP WITH TIME ZONE  NOT NULL
);
