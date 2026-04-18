# AGENTS

## Conventional Commits

This repository expects conventional commit style for both commit messages and pull request titles.

Allowed top-level types:

- `feat`
- `fix`
- `doc`
- `test`
- `ci`
- `refactor`
- `perf`
- `chore`
- `revert`
- `style`
- `security`

Examples:

- `feat(ui): add unified inbox layout`
- `fix(storage): handle sqlite migration failure`
- `doc(plans): split roadmap into numbered plans`
- `ci(release): upload desktop installers`

## Why It Matters

- `.github/workflows/commits.yml` validates pull request titles against the allowed conventional commit types.
- `cliff.toml` uses conventional commit prefixes to group changelog entries for releases.
- release automation depends on a reserved release commit and PR title format.

## Release Commit Notes

Do not manually use release commit titles for normal work.

Reserved automated release format:

- `chore(release): vX.Y.Z`

The release workflows use that exact pattern when creating the release branch, tagging merged release PRs, and skipping generated release commits in later changelogs.

## Practical Guidance

- Keep the subject line short and imperative.
- Use a scope when it clarifies the area being changed, such as `ui`, `storage`, `providers`, `plans`, or `release`.
- Prefer `doc` for documentation-only work and `ci` for workflow or automation changes.
- If a change spans multiple areas but is maintenance-oriented, `chore` is acceptable.
