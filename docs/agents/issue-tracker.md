# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues. Use the `gh` CLI for all operations.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."` or `--body-file` for longer content.
- **Read an issue**: `gh issue view <number> --comments` and include labels when needed.
- **List issues**: `gh issue list` with the state and label filters needed for the task.
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply or remove labels**: `gh issue edit <number> --add-label "..."` or `--remove-label "..."`
- **Close an issue**: `gh issue close <number> --comment "..."`

Infer the repo from `git remote -v`. `gh` does this automatically inside this clone.

## When a skill says "publish to the issue tracker"

Create a GitHub issue in this repository.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.
