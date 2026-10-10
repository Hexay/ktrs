package com.example

fun multi(a: Int, b: Int): Int {
    val list = listOf(a, b).map { it+1 }.filter { it > 0 }.map { it * 2 }.sortedBy { it }.first { it > 100 || it < -100 || it == 4 }
    if (a > b) return a; else return list
}

fun `bad name`() = Unit
val x = if (true) { 1 } else { 2 }
