# depub: minimise visibility

When working on medium or large sized Rust code bases, it can be hard to know
whether the visibility of functions, structs, and so on are still at the
minimum required. For example, sometimes functions that once needed to be `pub`
now only need to be `pub(crate)`.

`depub` minimises the visibility of such elements in a code base: in essence,
it does a string search for `pub`, replaces it with `pub crate` and sees if a
test command still succeeds. If it does, it keeps that visibility; otherwise it
tries the next in the list. As this suggests, `depub` is destructive: it
overwrites files, so do not run it on source code that you do not want altered!

For example, if you want to reduce visibility of a normal build, `cd` to your
Rust code base and execute:

```
$ /path/to/depub -- -c "cargo check"
```

The string specified with `-c` is passed straight to `/bin/sh` so can be any
shell command. If you want to test multiple features you might use:

```
$ /path/to/depub -- -c "cargo check && cargo check --features f1"
```

`depub` informs you of its progress. After it is finished, `diff` your code
base, and accept those of its recommendations you think appropriate.
