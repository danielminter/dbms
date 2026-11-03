# dbms

A disk-backed SQL database engine written from scratch in Rust.

## Architecture

SQL text → Tokenizer → Parser (builds an AST) → Executor → B+Tree → Pager (4 KB
pages on disk)

## Status

| Part                          | Status                 |
| ----------------------------- | ---------------------- |
| REPL shell (`.help`, `.exit`) | Done                   |
| Tokenizer                     | Done, with tests       |
| Parser                        | CREATE, SELECT, INSERT |
| Pager and cursor              | In progress            |
| Executor and B+Tree           | Planned                |

## Run

      cargo run

## Test

      cargo test
