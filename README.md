![CI](https://github.com/ant1k9/noter/workflows/tests/badge.svg)
[![codecov](https://codecov.io/gh/ant1k9/noter/branch/main/graph/badge.svg)](https://codecov.io/gh/ant1k9/noter)

### Noter

#### Usage

📝 Noter keeps your notes and allow you to filter them by tags. By default noter uses the Vim editor.

```bash
$ cargo install --path .
$ noter --help
$ noter init        # need to run before start using it
$ noter             # by default lists all notes
$ noter --tag test  # filter notes with given tag
$ noter add         # add new note
```

#### Roadmap

1. Use predefined markdown format for new notes ✅
2. Initialize folder where to save the data and metadata files ✅
3. Save all versions of notes to the same file ✅
4. List all notes ✅
5. Add remove command ✅
6. Add list filters ✅
