ALTER DATABASE spike SET timezone TO 'America/Sao_Paulo';
ALTER DATABASE spike SET datestyle TO 'ISO, DMY';
ALTER DATABASE spike SET default_text_search_config TO 'pg_catalog.portuguese';

CREATE DATABASE spike_testing OWNER spike;
