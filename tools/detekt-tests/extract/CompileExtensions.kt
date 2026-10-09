package dev.detekt.test.utils

import org.jetbrains.kotlin.psi.KtFile
import java.nio.file.Path
import kotlin.io.path.Path
import kotlin.io.path.absolute

// Stands in for detekt-test-utils' CompileExtensions.kt in tools/detekt-tests/extract-goldens.sh: the overloads
// syntax-only rule tests call, over the upstream KtTestCompiler. Left out: the Analysis API engine overload.

fun compileContentForTest(content: String, filename: String = "Test.kt"): KtFile {
    require('/' !in filename && '\\' !in filename) {
        "filename must be a file name only and not contain any path elements"
    }
    return compileContentForTest(content, path = Path("/").absolute().resolve(filename))
}

fun compileContentForTest(content: String, path: Path): KtFile = KtTestCompiler.createKtFile(content, path)

fun compileForTest(path: Path) = KtTestCompiler.compile(path)
