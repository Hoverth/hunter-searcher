--
-- PostgreSQL database dump
--

-- Dumped from database version 16.3 (Debian 16.3-1.pgdg120+1)
-- Dumped by pg_dump version 16.2

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: update_webpage_content(); Type: FUNCTION; Schema: public; Owner: crawler
--

CREATE FUNCTION public.update_webpage_content() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    new.search_vector := setweight(to_tsvector(coalesce(new.title, '')), 'A') ||
        setweight(to_tsvector(coalesce(new.blurb, '')), 'B') ||
        setweight(to_tsvector(coalesce(new.content, '')), 'C') ||
        setweight(to_tsvector(coalesce(new.url, '')), 'D');
    return new;
END
$$;


ALTER FUNCTION public.update_webpage_content() OWNER TO crawler;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: webpages; Type: TABLE; Schema: public; Owner: crawler
--

CREATE TABLE public.webpages (
    id integer NOT NULL,
    title text NOT NULL,
    blurb text,
    content text NOT NULL,
    number_js integer NOT NULL,
    url text NOT NULL,
    search_vector tsvector
);


ALTER TABLE public.webpages OWNER TO crawler;

--
-- Name: webpages_id_seq; Type: SEQUENCE; Schema: public; Owner: crawler
--

CREATE SEQUENCE public.webpages_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.webpages_id_seq OWNER TO crawler;

--
-- Name: webpages_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: crawler
--

ALTER SEQUENCE public.webpages_id_seq OWNED BY public.webpages.id;


--
-- Name: webpages id; Type: DEFAULT; Schema: public; Owner: crawler
--

ALTER TABLE ONLY public.webpages ALTER COLUMN id SET DEFAULT nextval('public.webpages_id_seq'::regclass);


--
-- Data for Name: webpages; Type: TABLE DATA; Schema: public; Owner: crawler
--

COPY public.webpages (id, title, blurb, content, number_js, url, search_vector) FROM stdin;
\.


--
-- Name: webpages_id_seq; Type: SEQUENCE SET; Schema: public; Owner: crawler
--

SELECT pg_catalog.setval('public.webpages_id_seq', 1, false);


--
-- Name: webpages webpages_pkey; Type: CONSTRAINT; Schema: public; Owner: crawler
--

ALTER TABLE ONLY public.webpages
    ADD CONSTRAINT webpages_pkey PRIMARY KEY (id);


--
-- Name: ix_search_vector; Type: INDEX; Schema: public; Owner: crawler
--

CREATE INDEX ix_search_vector ON public.webpages USING gin (search_vector);


--
-- Name: webpages webpage_search_vector_update; Type: TRIGGER; Schema: public; Owner: crawler
--

CREATE TRIGGER webpage_search_vector_update BEFORE INSERT OR UPDATE ON public.webpages FOR EACH ROW EXECUTE FUNCTION public.update_webpage_content();


--
-- PostgreSQL database dump complete
--

