fun quux() {
  runnnnn {
    foo()
    runnnnn {
      foo()
      bar()
    }
        .baz {
          foo()
          bar()
        }
  }
      .baz {
        foo()
        runnnnn {
          foo()
          bar()
        }
            .baz {
              foo()
              bar()
            }
      }
}
