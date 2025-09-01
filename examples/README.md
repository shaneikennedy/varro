# Varro examples

## Ingestion
You'll want to run any of these before trying the other examples.
- `ingest_flush_per_doc` calls `flush` after `index` for each doc to create a bunch of small segments. Use this if you want to see segment compaction in action.
- `ingest` One big flush for all documents in the documents directory

## Searching
- `search_and_retrieve` self explanitory, searches for a term and logs the results to the console
- `and_search_operator` demonstrates the AND behavior for multi term queries

## Webserver
- `webserver` how to use this in an actix_web api


## Config
- `with_config` an example of how to configure Varro


## Document
- `document` an example of the basic Document struct
