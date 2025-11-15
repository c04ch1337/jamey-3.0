# Memory System Rules
- Five distinct layers must maintain separate Tantivy indices
- Each memory layer gets its own subdirectory in data/
- Use UUID v4 for all memory record identifiers
- Store timestamps with chrono::Utc::now()
- Schema includes: id (STRING), content (TEXT), timestamp (DATE)
- Memory operations should be async where possible
