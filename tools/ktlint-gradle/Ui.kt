package com.example

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
fun myButton(text: String, modifier: Modifier) {
    println(text)
}

@Composable
fun Screen(modifier: Modifier = Modifier) {
    myButton("a", Modifier)
}
