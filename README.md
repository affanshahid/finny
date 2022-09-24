# Finny

A little CLI tool to view and aggregate financial transactions using the SMS messages sent by banks.
This **only works on macOS** as it reads messages from the Messages app, which can be configured to sync SMS messages from a paired iPhone.

## Configuration

Run `finny --help` to view usage details

```rs
finny 0.1.0
Calculate your expenses from messages sent by your bank

USAGE:
    finny [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -c, --config <CONFIG>
            Path to the matchers config [default: ./config.yml]

        --contacts <CONTACTS>
            Space separated list of contact numbers [default: 8012 9355]

    -e, --end <END>
            End date and time between which to perform analysis [default: "2022-09-24
            14:48:34.817398 UTC"]

        --exclude-sources <EXCLUDE_SOURCES>
            Sources to filter out [default: "JS Credit Card Bill Pay From IB"]

    -h, --help
            Print help information

    -s, --start <START>
            Start date and time between which to perform analysis [default: "2022-06-24
            14:48:34.817367 UTC"]

    -V, --version
            Print version information

SUBCOMMANDS:
    help             Print this message or the help of the given subcommand(s)
    subscriptions    Shows detected subscriptions from your data
    totals           Shows aggregated totals for each source
    transactions     Shows a table of transactions
```

Matchers are what finny uses to parse and understand messages. Check `example.config.yml` to get a better understanding.

## Development

### Requirements

- macOS
- Rust (MSRV 1.65.0)

### Building

```sh
# development
cargo build

# release
cargo build --release
```
