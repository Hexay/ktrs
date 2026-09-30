package com.example

// Assume that "com.example.Baz" defines a rangeUntil operator as follows:
//     operator fun Baz.rangeUntil(baz: Baz) = ...
import com.example.Baz.rangeUntil

val baz = Baz(1.0)..<Baz(2.0)