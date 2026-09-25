fun f() {
  someObject
      .someMethodReturningCollection()
      .map { it.someProperty }
      .find { it.contains(someSearchValue) }
      ?: someDefaultValue
}
