package com.example

import org.mockito.Mockito

fun foo() {
        Mockito.mock(String::class.java, Mockito.withSettings().defaultAnswer {  })
    }