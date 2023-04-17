# depub 0.1.1 (2023-04-17)

* Elements in top-level modules can't have `super` visibility: since these
  generally led to an error, depub wouldn't try private visibility in top-level
  modules. This is now fixed.


# depub 0.1.0 (2021-03-30)

* First release.
