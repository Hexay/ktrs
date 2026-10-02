fun f() {
    assertEquals(listOf(1), dest.select().where { true }.toList())
}
