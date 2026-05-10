# Conformance runner protocol

On-device conformance Apps should write line-oriented records to TT01 so a host runner can capture and classify results.

## Record format

```text
BADGEVMS_STD_TEST <suite> <case> PASS
BADGEVMS_STD_TEST <suite> <case> FAIL <message>
BADGEVMS_STD_TEST <suite> <case> SKIP <reason>
```

Rules:

- `<suite>` and `<case>` contain ASCII letters, digits, `_`, and `-` only.
- `PASS` records have no trailing message.
- `FAIL` records include a short diagnostic.
- `SKIP` records include the missing fixture or explicit BadgeVMS non-goal.
- A test App exits normally after printing all records.

## Suites

Recommended suites:

- `target`
- `link`
- `thread`
- `sync`
- `time`
- `fs`
- `stdio`
- `env`
- `process`
- `net`
- `unsupported`
- `fd`

The runner treats any unknown malformed line as a harness error, not as a test failure.
