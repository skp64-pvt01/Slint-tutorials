# Step 09 — Packaging and installation

**Level 4 · Advanced · ~40 minutes**

The application of step 08 runs from `target/release/`. Nobody can install it.
This step wraps the cargo build in a small CMake project that installs it and
packs it with **CPack**.

Cargo has no packaging of its own: `cargo install` copies a binary and nothing
else. A system package also needs a data file, a desktop entry, metadata and
dependencies — that is CPack's job, and CMake is the tool that drives both.

---

## What you will learn

| Piece | What it does |
|---|---|
| `add_custom_command` + `add_custom_target` | runs `cargo build --release` from `cmake --build` |
| `install(PROGRAMS ... RENAME notes)` | installs the binary under the name a user types |
| `install(FILES ...)` | the shortcuts data file and the `.desktop` entry |
| `CPACK_*` / `CPACK_DEBIAN_*` | name, version, contact, generated dependencies |
| `NOTES_DATA_DIR` | the data path compiled into the binary |

---

## Why this is a separate project

This folder contains no Rust source. It builds **step 08**:

```cmake
add_custom_command(
    OUTPUT "${NOTES_BINARY}"
    COMMAND ${CMAKE_COMMAND} -E env
        "CARGO_TARGET_DIR=${RUST_TARGET_DIR}"
        "NOTES_DATA_DIR=${CMAKE_INSTALL_FULL_DATADIR}/notes"
        "${CARGO}" build --release --manifest-path "${RUST_COURSE}/Cargo.toml"
            -p step-08-capstone-notes
    WORKING_DIRECTORY "${RUST_COURSE}")
add_custom_target(notes ALL DEPENDS "${NOTES_BINARY}")
```

`cmake --build` now runs cargo; a real project would put the same command in
its own `CMakeLists.txt`.

`NOTES_CARGO_TARGET_DIR` is a cache variable for cargo's output folder. It
defaults to `${CMAKE_CURRENT_BINARY_DIR}/cargo`; point it somewhere roomy when
the build folder is on a small `/tmp`.

---

## Two decisions that make the package work

**1. Where the data file lives.** Step 08 resolves `shortcuts.txt` in this
order: the `NOTES_DATA_DIR` environment variable, then the value compiled in
with `option_env!`, then `data/` next to the crate:

```rust
fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("NOTES_DATA_DIR") { return dir.into(); }
    match option_env!("NOTES_DATA_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"),
    }
}
```

`option_env!` is read **when the crate is compiled**, so the packaging build
passes it:

```cmake
"NOTES_DATA_DIR=${CMAKE_INSTALL_FULL_DATADIR}/notes"     # /usr/share/notes
```

**2. What the binary links.** Slint is compiled into the binary; there is no
`libslint.so` for the package to ship. What remains is libc, libgcc and the
font/windowing libraries the default backend pulled in. `SHLIBDEPS` finds them
and writes the `Depends:` line:

```cmake
set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
```

---

## The CMakeLists, part by part

```cmake
project(notes VERSION 1.0.0 LANGUAGES NONE)
include(GNUInstallDirs)
find_program(CARGO cargo REQUIRED)

# ... the add_custom_command above ...

install(PROGRAMS "${NOTES_BINARY}" DESTINATION ${CMAKE_INSTALL_BINDIR} RENAME notes)
install(FILES "${RUST_COURSE}/08-capstone-notes/data/shortcuts.txt"
    DESTINATION ${CMAKE_INSTALL_DATADIR}/notes)
install(FILES notes.desktop DESTINATION ${CMAKE_INSTALL_DATADIR}/applications)

set(CPACK_GENERATOR "TGZ")
if(UNIX AND NOT APPLE)
    set(CPACK_GENERATOR "TGZ;DEB")
    set(CPACK_DEBIAN_PACKAGE_NAME "notes")
    set(CPACK_DEBIAN_FILE_NAME DEB-DEFAULT)
    set(CPACK_DEBIAN_PACKAGE_MAINTAINER "Slint tutorials <you@example.com>")
    set(CPACK_DEBIAN_PACKAGE_SECTION "editors")
    set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
endif()
include(CPack)
```

`include(CPack)` must come last: it writes `CPackConfig.cmake` from the
variables above.

---

## Build the package

```bash
cd 09-packaging

cmake -S . -B /tmp/notes -DCMAKE_INSTALL_PREFIX=/usr \
  -DNOTES_CARGO_TARGET_DIR=/tmp/notes-cargo
cmake --build /tmp/notes              # runs cargo --release
cd /tmp/notes
cpack -G DEB
cpack -G TGZ
```

Expected:

```
CPack: - package: /tmp/notes/notes_1.0.0_amd64.deb generated.
```

Inspect it:

```bash
dpkg-deb -f notes_1.0.0_amd64.deb Package Version Depends
dpkg-deb -c notes_1.0.0_amd64.deb
```

On this machine the dependency line is:

```
Depends: libc6 (>= 2.43), libfontconfig1 (>= 2.12.6), libgcc-s1 (>= 4.2)
```

The exact list depends on the backend and the distribution; what matters is
that it names system libraries and no Slint library.

The contents:

```
/usr/bin/notes
/usr/share/notes/shortcuts.txt
/usr/share/applications/notes.desktop
```

---

## Install it

```bash
sudo apt install ./notes_1.0.0_amd64.deb
which notes
notes --test
dpkg -L notes
sudo apt remove notes
```

`notes --test` is the check from step 08, now running from `/usr/bin/notes` and
reading `/usr/share/notes/shortcuts.txt` for the Help page.

---

## The archive, and staging an install

```bash
cpack -G TGZ
tar xzf notes-1.0.0-Linux.tar.gz -C /tmp/notes-1.0.0
NOTES_DATA_DIR=/tmp/notes-1.0.0/usr/share/notes \
  /tmp/notes-1.0.0/usr/bin/notes
```

The `NOTES_DATA_DIR` line is needed because the compiled data path still says
`/usr/share/notes`. Configure with the prefix you will really use
(`-DCMAKE_INSTALL_PREFIX=$HOME/.local`) if you want a self-consistent archive.

To look at an install without doing it:

```bash
DESTDIR=/tmp/stage cmake --install /tmp/notes
find /tmp/stage -type f
```

`cpack` uses exactly this staging internally. A DEB can be unpacked the same
way:

```bash
dpkg-deb -x notes_1.0.0_amd64.deb /tmp/stage
/tmp/stage/usr/bin/notes --test
```

---

## Where the files go

| Path | Content |
|---|---|
| `/usr/bin/notes` | the compiled binary |
| `/usr/share/notes/shortcuts.txt` | the help data, read-only |
| `/usr/share/applications/notes.desktop` | menu entry |

Keep user data out of `/usr`: settings belong in the user's configuration
folder, documents wherever the user saves them.

---

## Checkpoint

* What does `option_env!` read, and when?
* Why is there no Slint library in `Depends:`?
* What runs when `cmake --build` is called on this project?
* Which CPack variable produces `Depends:` without listing libraries by hand?
* What does `RENAME notes` change?

---

## Exercises

1. Add a man page (`install(FILES notes.1 DESTINATION ${CMAKE_INSTALL_MANDIR}/man1)`).
2. Bump `project(... VERSION)` and repackage; check the file name.
3. Add the `notes.desktop` entry to the package's `MimeType` handling, or add
   an icon and install it under `${CMAKE_INSTALL_DATADIR}/icons`.
4. Configure with `-DCMAKE_INSTALL_PREFIX=$HOME/.local`, install without sudo,
   and run the binary. Which path did `NOTES_DATA_DIR` get?
5. Add a CPack component (`COMPONENTS`) that splits the binary and the data.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `cpack` says the file does not exist | `cmake --build` was not run, so cargo never produced the binary. |
| The installed app cannot find `shortcuts.txt` | `NOTES_DATA_DIR` was compiled from a different prefix. Reconfigure with `-DCMAKE_INSTALL_PREFIX=/usr`, or set `NOTES_DATA_DIR` at run time. |
| `Depends` lists a Slint library | Slint is static in the binary; this would mean a custom dynamic build. The default Cargo build needs nothing extra. |
| `dpkg-shlibdeps` fails | It is part of `dpkg-dev`; install it, or turn `SHLIBDEPS` off and list the dependencies yourself. |
| The binary is called `08-capstone-notes` after installing | `RENAME notes` is missing from `install(PROGRAMS ...)`. |
| `No space left on device` during the release build | Point `NOTES_CARGO_TARGET_DIR` at a roomy filesystem. |

**Previous:** [Step 08 — Capstone: Notes](../08-capstone-notes/README.md) ·
**Back to the course index:** [README](../README.md)
