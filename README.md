# depub: minimise visibility

## Overview

When working on medium or large sized Rust code bases, it can be hard to know
whether the visibility of functions, structs, and so on are still at the
minimum required. For example, sometimes functions that once needed to be `pub`
now only need to be `pub(crate)`, `pub(super)`, or simply private.

`depub` minimises the visibility of such items in files passed to it, using a
user-specified command (e.g. `cargo check`) as an oracle to tell if its
reduction of an item's visibility is valid or not. Note that `depub` is
entirely guided by the oracle command: if the code it compiles happens not to
use part of an intentionally public interface, then `depub` is likely to
suggest reducing its visibility even though that's not what you want. The
broader the coverage of your oracle, the less this is an issue.

In essence, `depub` does a string search for `pub`, replaces it with `pub
crate` and sees if a test command still succeeds. If it does, it keeps that
visibility, otherwise it replaces with the original and tries the next item.
Note that `depub` is inherently destructive: it overwrites files as it
operates, so do not run it on source code that you do not want altered!

The list of visibilities that `depub` considers is, in order: `pub`,
`pub(crate)`, `pub(super)`, and private (i.e. no `pub` keyword at all). `depub`
searches for `pub`/`pub(crate)`/`pub(super)` instances, reduces their
visibility by one level, and tries the oracle command. If it succeeds, it tries
the next lower level until private visibility has been reached.

Since reducing the visibility of one item can enable other items' visibility to
be reduced, `depub` keeps running "rounds" until a fixed point has been
reached. The maximum number of rounds is equal to the number of visible items
in the code base, though in practise 2 or 3 rounds are likely to be all that is
needed.


## Usage

`depub`'s usage is as follows:

```
depub -c <command> file_1 [... file_n]
```

where `<command>` is a string to be passed to `/bin/sh -c` for execution to
determine whether the altered source code is still valid.

To reduce the visibility of a normal Rust project, `cd` to your Rust code base
and execute:

```
$ find . -name "*.rs" | \
    xargs /path/to/depub -c "cargo check && cargo check --test"
```

`depub` informs you of its progress. After it is finished, `diff` your code
base, and accept those of its recommendations you think appropriate. Note that
`depub` currently uses string search and replace, so it will merrily change the
string `pub` in a command into `pub(crate)` -- you should not expect to accept
its recommendations without at least a cursory check.


## Using with libraries

Running `depub` on a library will tend to reduce all its intentionally `pub`
functions to private visibility. You can weed these out manually after `depub`
has run, but this can be tedious, and may also have reduced the visibility of a
cascade of other items.

To avoid this, use one or more users of the library in the oracle command as part
of your oracle. Temporarily alter their `Cargo.toml` to point to the local
version of your libary and use a command such as:

```
$ find . -name "*.rs" | \
    xargs /path/to/depub -c " \
      cargo check && cargo check --test && \
      cd /path/to/lib && cargo check && cargo check --test"
```
