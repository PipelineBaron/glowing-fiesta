# Glowing Fiesta

## Wild assumptions

I think there was some natural ambiguity built into this. The biggest ambiguous
thing that stood out to me was specifics around what type of transaction disputes
can be applied to. Personally, it didn't make a ton of sense to me that someone
would dispute a deposit into their own account (a withdrawal against their asset
account). So in the current implementation, all disputes that don't target deposit
transactions are considered errors and ignored with proper error notification. It
might be the case that this was an invalid assumption, but if it was, it's a quick
fix to implement.

## Overview

The main component of the system is the `LedgerSystem`. For its inputs, it takes
a `Ledger` which represents the state of the system, an input stream and an output
stream. The input stream is expected to produce a CSV compatible with the supplied
specification, and the output stream is where the resulting account CSV will be
written out to.

The input stream supplied to the `LedgerSystem` is wrapped in a `TransactionReader`
which parses the CSV data from the stream and produces a streaming iterator
that takes the CSV rows deserialized with `serde` and further refined into more
type restricted types that have less variance with a single `TryFrom` for validation.
The resulting iterator is one that produces elements of the `Transaction`.

The `LedgerSystem` then takes the iterator of `Transaction`s and applies all the
transactions one at a time to the `Ledger` state. The `Ledger` struct contains a
state machine that knows how to apply transactions to itself. The CSV is assumed
to be put together correctly and in good faith by our partner, so when we encounter
errors during transaction application, we log them to stdout individually and continue
processing transactions.

Once all transactions have been applied to the `Ledger`, the `Ledger` writes the state
all of its accounts out to the supplied output stream as a CSV in the formatted per
the specification.

## Testing

This project contains unit tests, integration tests and a manual runnable test. The
unit tests and integration tests are runnable via the usual `cargo test`. The manual
test is runnable via `cargo run -- transactions.csv > accounts.csv`.

### Unit Tests

Not every single component of the system has unit tests, but the ones I thought were
most critical to get right do. Notable standouts are `TransactionReader`,
and `AccountState`.

### Integration Tests

Integration tests can be found under `src/` they all operate under the same basic
framework of preparing a well-formatted CSV in memory and running it through a fresh
`LedgerSystem` configured to write its final account state CSV to a memory-based
stream and makes assertions on the resulting account state CSV