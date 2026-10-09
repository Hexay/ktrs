package io.github.hexay.ktrs.intellij

import com.intellij.codeInsight.daemon.impl.DaemonCodeAnalyzerImpl
import com.intellij.codeInsight.daemon.impl.HighlightInfo
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.redhat.devtools.lsp4ij.ConnectDocumentToLanguageServerSetupParticipant
import com.intellij.openapi.fileTypes.FileTypeManager
import com.intellij.openapi.fileTypes.PlainTextFileType
import com.intellij.openapi.application.runWriteAction
import com.intellij.psi.codeStyle.CodeStyleManager
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.UsefulTestCase
import com.intellij.testFramework.builders.EmptyModuleFixtureBuilder
import com.intellij.testFramework.fixtures.CodeInsightTestFixture
import com.intellij.testFramework.fixtures.IdeaTestFixtureFactory
import com.intellij.testFramework.fixtures.impl.CodeInsightTestFixtureImpl
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
        // Plain text: keeps the bundled Kotlin plugin's analysis out of the highlights.
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
        fileOpened(fixture.project, fixture.file.virtualFile)
        val warnings = await("the no-multi-spaces diagnostic") {
            highlights().takeIf { infos -> infos.any(::isNoMultiSpaces) }
        }
        assertEquals(HighlightSeverity.WARNING, warnings.first(::isNoMultiSpaces).severity)

        WriteCommandAction.runWriteCommandAction(fixture.project) { CodeStyleManager.getInstance(fixture.project).reformat(fixture.file) }
        val formatted = await("the formatted document") { fixture.editor.document.text.takeIf { it == "fun f() = 1\n" } }
        assertEquals("fun f() = 1\n", formatted)
    }

    protected open fun fileOpened(project: Project, file: VirtualFile) {}

    /** The platform client's annotator results plus what LSP4IJ applies straight to the markup model. */
    private fun highlights(): List<HighlightInfo> {
        // canChangeDocument: the platform client restarts the daemon when diagnostics arrive, mid-highlighting.
        val passes = CodeInsightTestFixtureImpl.instantiateAndRun(fixture.file, fixture.editor, IntArray(0), true)
        return passes + DaemonCodeAnalyzerImpl.getHighlights(fixture.editor.document, null, fixture.project)
    }

    private fun isNoMultiSpaces(info: HighlightInfo) = "Unnecessary long whitespace" in info.description.orEmpty()

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

class Lsp4ijIntegrationTest : KtrsIntegrationTest(KtrsClients.LSP4IJ) {
    // LSP4IJ subscribes to fileOpened in projectOpened, which the test project may fire after the fixture opens the file.
    override fun fileOpened(project: Project, file: VirtualFile) {
        ConnectDocumentToLanguageServerSetupParticipant().fileOpened(FileEditorManager.getInstance(project), file)
    }
}
