package io.github.ktlint.core
import io.github.ktlint.core.enum.Enum
import io.github.ktlint.core.enum.EnumThree
import io.github.ktlint.core.enum.EnumTwo
data class TrailingCommaTest(
    val foo: String,
    val bar: Enum,
    val bar2: EnumTwo,
    val bar3: EnumThree
)