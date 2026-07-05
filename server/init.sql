
CREATE SEQUENCE public.user_variable_id_seq;

CREATE TABLE public.user_variable (
                id INTEGER NOT NULL DEFAULT nextval('public.user_variable_id_seq'),
                id_str VARCHAR(100) NOT NULL,
                value VARCHAR,
                CONSTRAINT user_variable_pk PRIMARY KEY (id)
);


ALTER SEQUENCE public.user_variable_id_seq OWNED BY public.user_variable.id;

CREATE UNIQUE INDEX user_variable_id_str_unique
 ON public.user_variable
 ( id_str );

CREATE SEQUENCE public.server_id_seq;

CREATE TABLE public.server (
                id INTEGER NOT NULL DEFAULT nextval('public.server_id_seq'),
                id_str VARCHAR(100) NOT NULL,
                name VARCHAR(100) NOT NULL,
                description VARCHAR(1000) DEFAULT 'No description' NOT NULL,
                ip VARCHAR(15) NOT NULL,
                api_key VARCHAR(36),
                disabled BOOLEAN NOT NULL,
                reverse_api_key VARCHAR(36),
                CONSTRAINT server_pk PRIMARY KEY (id)
);


ALTER SEQUENCE public.server_id_seq OWNED BY public.server.id;

CREATE UNIQUE INDEX api_key_server_unique
 ON public.server
 ( api_key );

CREATE SEQUENCE public.service_id_seq;

CREATE TABLE public.service (
                id INTEGER NOT NULL DEFAULT nextval('public.service_id_seq'),
                server_id INTEGER,
                id_str VARCHAR(100) NOT NULL,
                name VARCHAR(100) NOT NULL,
                last_config VARCHAR,
                disabled BOOLEAN NOT NULL,
                CONSTRAINT service_pk PRIMARY KEY (id)
);


ALTER SEQUENCE public.service_id_seq OWNED BY public.service.id;

CREATE UNIQUE INDEX service_id_str_unique
 ON public.service
 ( id_str );

CREATE SEQUENCE public.user_var_needs_update_id_seq;

CREATE TABLE public.user_var_needs_update (
                id INTEGER NOT NULL DEFAULT nextval('public.user_var_needs_update_id_seq'),
                service_id INTEGER NOT NULL,
                user_variable_id INTEGER NOT NULL,
                last_value VARCHAR,
                CONSTRAINT user_var_needs_update_pk PRIMARY KEY (id)
);


ALTER SEQUENCE public.user_var_needs_update_id_seq OWNED BY public.user_var_needs_update.id;

CREATE SEQUENCE public.ip_needs_update_id_seq;

CREATE TABLE public.ip_needs_update (
                id INTEGER NOT NULL DEFAULT nextval('public.ip_needs_update_id_seq'),
                service_trigger_id INTEGER NOT NULL,
                service_updated_id INTEGER NOT NULL,
                last_ip VARCHAR(15) NOT NULL,
                CONSTRAINT ip_needs_update_pk PRIMARY KEY (id)
);


ALTER SEQUENCE public.ip_needs_update_id_seq OWNED BY public.ip_needs_update.id;

ALTER TABLE public.user_var_needs_update ADD CONSTRAINT user_variable_user_var_needs_update_fk
FOREIGN KEY (user_variable_id)
REFERENCES public.user_variable (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE public.service ADD CONSTRAINT server_service_fk
FOREIGN KEY (server_id)
REFERENCES public.server (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE public.ip_needs_update ADD CONSTRAINT service_needs_update_fk
FOREIGN KEY (service_trigger_id)
REFERENCES public.service (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE public.ip_needs_update ADD CONSTRAINT service_needs_update_fk1
FOREIGN KEY (service_updated_id)
REFERENCES public.service (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE public.user_var_needs_update ADD CONSTRAINT service_user_var_needs_update_fk
FOREIGN KEY (service_id)
REFERENCES public.service (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;
