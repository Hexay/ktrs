package io.github.hexay.ktrs.intellij

import com.intellij.codeInsight.daemon.impl.HighlightInfo
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.fileTypes.FileTypeManager
import com.intellij.openapi.fileTypes.PlainTextFileType
import com.intellij.openapi.application.runWriteAction
import com.intellij.psi.codeStyle.CodeStyleManager
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.UsefulTestCase
import com.intellij.testFramework.builders.EmptyModuleFixtureBuilder
import com.intellij.testFramework.fixtures.CodeInsightTestFixture
import com.intellij.testFramework.fixtures.IdeaTestFixtureFactory
import com.intellij.testFramework.fixtures.impl.TempDirTestFixtureImpl
import java.nio.file.Files
import java.nio.file.Path

/**
 * Opens a Kotlin file with ktlint violations in a project on disk, runs the real `ktrs lsp` (`-PktrsExecutable`)
 * through one LSP client, and checks a diagnostic arrives and Reformat Code formats with ktfmt.
 */
abstract class KtrsIntegrationTest(private val client: String) : UsefulTestCase() {
    private lateinit var fixture: CodeInsightTestFixture

    override fun setUp() {
        super.setUp()
        System.setProperty(KtrsClients.PROPERTY, client)
        val factory = IdeaTestFixtureFactory.getFixtureFactory()
        val builder = factory.createFixtureBuilder(name)
        fixture = factory.createCodeInsightFixture(builder.fixture, TempDirTestFixtureImpl())
        builder.addModule(EmptyModuleFixtureBuilder::class.java).addContentRoot(fixture.tempDirPath)
        fixture.setUp()
        val executable = System.getProperty("ktrs.testExecutable")
        assertTrue("no ktrs binary at $executable (cargo build --bins, or -PktrsExecutable)", Files.isRegularFile(Path.of(executable)))
        KtrsLocalSettings.getInstance(fixture.project).state.path = executable
        KtrsSettings.getInstance(fixture.project).state.apply {
            formatTool = "ktfmt"
            ktfmtStyle = "kotlinlang"
        }
        // The test IDE loads no Kotlin plugin.
        runWriteAction { FileTypeManager.getInstance().associateExtension(PlainTextFileType.INSTANCE, "kt") }
    }

    override fun tearDown() {
        try {
            fixture.tearDown()
        } finally {
            System.clearProperty(KtrsClients.PROPERTY)
            super.tearDown()
        }
    }

    fun testDiagnosticsAndFormatting() {
        fixture.configureByText("Main.kt", "fun  f( ) = 1\n")
        val warnings = await("a ktlint diagnostic") { fixture.doHighlighting(HighlightSeverity.WEAK_WARNING).filter(::fromKtrs).ifEmpty { null } }
        assertTrue(warnings.joinToString { it.description }, warnings.any { "no-multi-spaces" in it.description.orEmpty() || "space" in it.description.orEmpty() })

        WriteCommandAction.runWriteCommandAction(fixture.project) { CodeStyleManager.getInstance(fixture.project).reformat(fixture.file) }
        val formatted = await("the formatted document") { fixture.editor.document.text.takeIf { it == "fun f() = 1\n" } }
        assertEquals("fun f() = 1\n", formatted)
    }

    private fun fromKtrs(info: HighlightInfo) = info.severity >= HighlightSeverity.WEAK_WARNING && !info.description.isNullOrEmpty()

    private fun <T : Any> await(what: String, probe: () -> T?): T {
        val deadline = System.nanoTime() + 60_000_000_000
        while (true) {
            PlatformTestUtil.dispatchAllEventsInIdeEventQueue()
            probe()?.let { return it }
            if (System.nanoTime() > deadline) {
                fail("$what didn't arrive in 60 s via $client; document: ${fixture.editor.document.text}")
            }
            Thread.sleep(100)
        }
    }
}

class PlatformLspIntegrationTest : KtrsIntegrationTest(KtrsClients.PLATFORM)

class Lsp4ijIntegrationTest : KtrsIntegrationTest(KtrsClients.LSP4IJ)
