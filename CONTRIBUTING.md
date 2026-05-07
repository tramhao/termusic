# Contributing

## Issue reports

Open an issue when:

- You have a bug to report.
- You have a suggestion on how to improve the project. (Consider opening a discussion instead)

## Discussions

Open a discussion if:

- You have questions or concerns regarding the project or the application itself. (Use Category `Q&A`)
- You have a idea to extend the project or a new feature idea. (Use Categroy `Ideas`)

## Contribuing code

### Commits

For a commit, we expect the code:

- Builds should not fail with the default features. (or the features this code touches)
- Properly formatted code. (via `cargo +nightly fmt`)
- No Clippy warnings. (via `cargo clippy`)
- To follow the general coding style of surrounding code.
- To not add code which would have a conflicting License.

#### Size of the commits

Commit should aim to be minimal and only a single feature related.

Targeted / minimal commits help in:

- review (PRs / `git blame`)
- debugging (`git bisect`)
- changelog generation
- easier changes in case a PR needs to be rebased
- revertion

TL;DR: commit often, not all at once.

### Commit messages

Commit messages are expected to be [conventional-commits](https://www.conventionalcommits.org/en/v1.0.0/).

Specifically, we use:

1. For types:
   - `feat`: Feature work, which would not fall into `refactor`, `fix` or `style`.
   - `fix`: A minor fix, which does not change much overall.
   - `refactor`: A Code refactor that does not change the observable behavior of the code.
   - `style`: A style only change. (ex. `cargo clippy --fix`)
   - `deps`: A Dependency update. (includes updates necessary for breaking dependency updates)
   - `docs`: Documentation update. (ex. for `README`; code should use `style` instead)
   - `chore`: Anything that does not touch the code itself and does not fall into any other category like `docs`.
   - `revert`: A Revert commit. (may be excempt from following conventional commits)
   - `merge`: A Merge commit. (may be excempt from following conventional commits)
2. For Scope:
   - for `deps`, use the dependency that is updated
   - for `chore` and `docs`, use the basename of the file (ex. `CODE_OF_CONDUCT`), but include extension or a path if necesary. (ex. `Cargo.toml`; `workflows/release`)
   - for code (`feat`, `fix`, `refactor`, `style`), use the common ancestor *module* path that is modified. (ex. on a single file `tui::ui::components::lyric`; many changes in a package `tui`) If there is no common ancestor (ex. changes across `lib` and `tui`), use `tui & lib` or dont use a scope.
3. For description:
   - include a simple summary for the changes; anything more extensive should use the body
4. For footer:
   - include `BREAKING CHANGE:` if the change is a observable breaking change (ex. removing support for a config version)
   - include references to the issue / PR this commit handles (ex. `fixes #1`; `re #1`)

Please keep in mind the project is `termusic` which is not meant to be consumed outside of `termusic`(TUI) and `termusic-server`, so any "observable" change is a change that the user would notice (ex. Changing the config; fixing TUI text).

Note that for simple changes, squash merging may be used and the commit message reworded.

### Pull requests

Pull requests are expected to:

- target the default (`master`) branch, unless specified otherwise.
- pass *all* github CI actions, which indlues all things from [Commits](#commits) and passing of all tests.
