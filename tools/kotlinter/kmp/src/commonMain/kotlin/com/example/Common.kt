package com.example

expect fun platform():String

class Common{
    fun  greet( name:String ) = "hi $name from ${platform()}"
}
