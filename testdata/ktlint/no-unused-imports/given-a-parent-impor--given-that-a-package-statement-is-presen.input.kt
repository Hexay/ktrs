package com.example

import org.mockito.Mockito
import org.mockito.Mockito.mock1
import org.mockito.Mockito.withSettings

fun foo() {
        Mockito.mock(String::class.java, Mockito.withSettings().defaultAnswer {  })
    }