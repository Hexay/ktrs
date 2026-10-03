package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import java.io.File
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.testdsl.PLUGIN_ID
import org.jlleitschuh.gradle.ktlint.testdsl.TestProject
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.buildAndFail
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test

class GitHookTasksTest : AbstractPluginTest() {

    @DisplayName("Should not add install git hook task to submodule")
    @Test
    fun shouldNotAddInstallTaskToSubmodule() {
        project {
            val submoduleDir = projectPath.resolve("some-module").also { it.mkdirs() }
            projectPath.initGit()
            settingsGradle.appendText("\ninclude \":some-module\"\n")
            submoduleDir.buildFile().writeText(
                "plugins {\n    id \"org.jetbrains.kotlin.jvm\"\n    id \"$PLUGIN_ID\"\n}\n"
            )

            for (taskName in listOf(INSTALL_GIT_HOOK_CHECK_TASK, INSTALL_GIT_HOOK_FORMAT_TASK)) {
                buildAndFail(":some-module:$taskName") {
                    assertThat(output.lowercase()).contains("task '$taskName' not found in project".lowercase())
                }
            }
        }
    }

    @DisplayName("Running install git hook check task should create pre-commit hook")
    @Test
    fun installPreCommitHookCheck() {
        project { assertInstallsHook(projectPath.initGit(), INSTALL_GIT_HOOK_CHECK_TASK, CHECK_PARENT_TASK_NAME) }
    }

    @DisplayName("Running install git hook when hooks dir doesn't exist check task should create pre-commit hook")
    @Test
    fun installPreCommitHookWithoutHooksDirCheck() {
        project {
            assertInstallsHook(projectPath.initGitWithoutHooksDir(), INSTALL_GIT_HOOK_CHECK_TASK, CHECK_PARENT_TASK_NAME)
        }
    }

    @DisplayName("Running install git hook format task should create pre-commit hook")
    @Test
    fun installPreCommitHookFormat() {
        project { assertInstallsHook(projectPath.initGit(), INSTALL_GIT_HOOK_FORMAT_TASK, FORMAT_PARENT_TASK_NAME) }
    }

    @DisplayName("Should find git folder if Gradle project is not located in root git working dir")
    @Test
    fun findGitDir() {
        val gradleRoot = projectRoot.resolve("internal/").also { it.mkdirs() }
        val gitDir = projectRoot.initGit()

        project(projectPath = gradleRoot) {
            assertInstallsHook(gitDir, INSTALL_GIT_HOOK_CHECK_TASK, CHECK_PARENT_TASK_NAME)
            assertThat(gitDir.preCommitGitHook().readText()).contains("./internal/gradlew -p ./internal")
        }
    }

    private fun TestProject.assertInstallsHook(gitDir: File, installTask: String, ktlintTask: String) {
        build(":$installTask") {
            assertThat(task(":$installTask")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            assertThat(gitDir.preCommitGitHook().exists()).isTrue()
            assertThat(gitDir.preCommitGitHook().canExecute()).isTrue()
            assertThat(gitDir.preCommitGitHook().readText()).contains(ktlintTask)
        }
    }

    @DisplayName("Should produce same hook on second run")
    @Test
    fun sameHookOnSecondRun() {
        project {
            val gitDir = projectPath.initGit()

            build(":$INSTALL_GIT_HOOK_FORMAT_TASK")
            val hookFileContent = gitDir.preCommitGitHook().readText()
            build(":$INSTALL_GIT_HOOK_FORMAT_TASK")
            assertThat(gitDir.preCommitGitHook().readText()).isEqualTo(hookFileContent)
        }
    }

    @DisplayName("Should not touch already existing hooks")
    @Test
    fun notTouchExistingHooks() {
        project {
            val gitDir = projectPath.initGit()
            gitDir.preCommitGitHook().writeText(
                "$shShebang\n\necho \"test1\"\n$startHookSection\n\n\n$endHookSection\necho \"test2\""
            )

            build(":$INSTALL_GIT_HOOK_FORMAT_TASK")

            val hookContent = gitDir.preCommitGitHook().readText()
            assertThat(hookContent).startsWith("$shShebang\n\necho \"test1\"")
            assertThat(hookContent).endsWith("echo \"test2\"")
        }
    }

    @DisplayName("Check hook should not include files into git commit")
    @Test
    fun checkHookShouldNotIncludeFilesIntoGitCommit() {
        assertHook(INSTALL_GIT_HOOK_CHECK_TASK) { assertThat(it).doesNotContain("git add") }
    }

    @DisplayName("Collects check and format run exit codes and uses them to indicate success")
    @Test
    fun hooksUseGradleExitCode() {
        for (installTask in listOf(INSTALL_GIT_HOOK_CHECK_TASK, INSTALL_GIT_HOOK_FORMAT_TASK)) {
            assertHook(installTask) { hookText ->
                val lines = hookText.lines()
                assertThat(lines).doesNotContain("set -e")
                assertThat(lines).contains("gradle_command_exit_code=\$?")
                assertThat(lines).contains("exit \$gradle_command_exit_code")
            }
        }
    }

    @DisplayName("Format hook should include updated files into git commit")
    @Test
    fun formatIncludeUpdatedFiles() {
        assertHook(INSTALL_GIT_HOOK_FORMAT_TASK) { assertThat(it).contains("git add") }
    }

    @DisplayName("Format hook should not add non-indexed code to the commit")
    @Test
    fun formatHookNonIndexedCode() {
        assertHook(INSTALL_GIT_HOOK_FORMAT_TASK) { hookText ->
            assertThat(hookText).contains("git diff --binary --color=never > \$diff")
            assertThat(hookText).contains("git apply -R \$diff")
            assertThat(hookText).contains("git apply --ignore-whitespace \$diff")
            assertThat(hookText).contains("rm \$diff")
        }
    }

    @DisplayName("Format hook should format renamed files, and only .kt and .kts files")
    @Test
    fun formatHookFileSelection() {
        assertHook(INSTALL_GIT_HOOK_FORMAT_TASK) { hookText ->
            assertThat(hookText).contains("""{ print $NF }""")
            assertThat(hookText).contains("""/\.kts?$/""")
        }
    }

    private fun assertHook(installTask: String, assertions: (String) -> Unit) {
        val root = temporaryFolder.resolve(installTask).also { it.mkdirs() }
        project(projectPath = root) {
            val gitDir = projectPath.initGit()
            build(":$installTask") {
                assertThat(task(":$installTask")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertions(gitDir.preCommitGitHook().readText())
            }
        }
    }

    private fun File.preCommitGitHook(): File = resolve("hooks/pre-commit")
}
