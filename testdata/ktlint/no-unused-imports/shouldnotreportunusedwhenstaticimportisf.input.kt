package com.foo

import android.text.Spannable
import androidx.core.text.toSpannable

fun foo(text: String): Spannable {
    return text.toSpannable()
}