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

CREATE SEQUENCE billing_service_id_seq START 1;
CREATE SEQUENCE billing_bill_id_seq START 1;
CREATE SEQUENCE billing_billline_id_seq START 1;
CREATE SEQUENCE billing_userprofile_id_seq START 1;


CREATE TABLE billjobs_service (
	id int4 DEFAULT nextval('billing_service_id_seq'::regclass) NOT NULL,
	reference varchar(5) NOT NULL,
	"name" varchar(128) NOT NULL,
	description varchar(256) NOT NULL,
	price float8 NOT NULL,
	is_available bool NOT NULL,
	CONSTRAINT billing_service_pkey PRIMARY KEY (id)
);

CREATE TABLE billjobs_bill (
	id int4 DEFAULT nextval('billing_bill_id_seq'::regclass) NOT NULL,
	user_id int4 NOT NULL,
	"number" varchar(16) NOT NULL,
	"isPaid" bool NOT NULL,
	billing_date date NOT NULL,
	amount float8 NOT NULL,
	issuer_address varchar(1024) NOT NULL,
	billing_address varchar(1024) NOT NULL,
	CONSTRAINT billing_bill_number_key UNIQUE (number),
	CONSTRAINT billing_bill_pkey PRIMARY KEY (id),
	CONSTRAINT billjobs_bill_user_id_07d2e338_fk_auth_user_id FOREIGN KEY (user_id) REFERENCES auth_user(id) DEFERRABLE INITIALLY DEFERRED
);
CREATE INDEX billing_bill_number_like ON billjobs_bill USING btree (number varchar_pattern_ops);
CREATE INDEX billing_bill_user_id ON billjobs_bill USING btree (user_id);

CREATE TABLE billjobs_billline (
	id int4 DEFAULT nextval('billing_billline_id_seq'::regclass) NOT NULL,
	bill_id int4 NOT NULL,
	service_id int4 NOT NULL,
	quantity int2 NOT NULL,
	total float8 NOT NULL,
	note varchar(1024) NOT NULL,
	CONSTRAINT billing_billline_pkey PRIMARY KEY (id),
	CONSTRAINT billjobs_billline_bill_id_099995ee_fk_billjobs_bill_id FOREIGN KEY (bill_id) REFERENCES billjobs_bill(id) DEFERRABLE INITIALLY DEFERRED,
	CONSTRAINT billjobs_billline_service_id_27f2aa51_fk_billjobs_service_id FOREIGN KEY (service_id) REFERENCES billjobs_service(id) DEFERRABLE INITIALLY DEFERRED
);
CREATE INDEX billing_billline_bill_id ON billjobs_billline USING btree (bill_id);
CREATE INDEX billing_billline_service_id ON billjobs_billline USING btree (service_id);

CREATE TABLE billjobs_userprofile (
	id int4 DEFAULT nextval('billing_userprofile_id_seq'::regclass) NOT NULL,
	user_id int4 NOT NULL,
	billing_address text NOT NULL,
	CONSTRAINT billing_userprofile_pkey PRIMARY KEY (id),
	CONSTRAINT billing_userprofile_user_id_key UNIQUE (user_id),
	CONSTRAINT billjobs_userprofile_user_id_8f0870f3_fk_auth_user_id FOREIGN KEY (user_id) REFERENCES auth_user(id) DEFERRABLE INITIALLY DEFERRED
);