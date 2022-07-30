# Search Syntax

A search query contains zero or more search terms, separated by spaces.

If any positive lexeme term is specified, results are ordered by relevance for the PostgreSQL backend.
If no positive lexeme term is specified, results are ordered by their creation time (newer notes come first) regardless
the backend.
In any other case, the order is unspecified.

If no positive lexeme term is specified, results are limited to 10 notes by default, unless a `!limit=<integer>`
modifier is used.

## Lexeme Terms

- Positive lexeme term: a plain word, such as `token`, will be searched against the title and the note body.
- Negative lexeme term: prefix a word with `-` to exclude the word, such as `-exclude`.

## Tag Terms

- Positive tag term: a hashtag, such as `#token`.
- Negative tag term: prefix a hashtag with `-` to exclude the tag, such as `-#exclude`.

## Modifier Terms

- `!notag`: match notes with no tags.
- `!orphan`: match notes that have no previous note, no parent note (i.e., not a branch of another note), and not
  referenced by other notes.
- `!limit=<integer>`: control the number of notes returned in the result to be `<integer>`.