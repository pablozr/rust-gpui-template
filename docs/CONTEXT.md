# Context

One line per domain term; code uses these words.

- **Item**: one entry of the list, with an id, a title and a done flag.
- **Store**: the port through which the core loads and saves items; implemented by the JSON file adapter and by the in-memory fake.
- **Problem**: a failure a person can be told about, with a message in `i18n::problems` and a flag for whether trying again can help.
