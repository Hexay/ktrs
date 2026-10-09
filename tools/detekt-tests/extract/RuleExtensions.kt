package dev.detekt.test

import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import dev.detekt.api.RuleName
import dev.detekt.test.utils.compileContentForTest
import org.jetbrains.kotlin.config.LanguageVersionSettings
import org.jetbrains.kotlin.psi.KtAnnotated
import org.jetbrains.kotlin.psi.KtAnnotationEntry
import org.jetbrains.kotlin.psi.KtElement
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

// Stands in for detekt-test's RuleExtensions.kt in tools/detekt-tests/extract-goldens.sh: the same `Rule.lint`
// entry points for syntax-only rules, with every `visitFile` result handed to CaseRecorder. Left out: snippet
// compilation (`compile-test-snippets`) and `lintWithContext` (Analysis API rules are not ported).

fun Rule.lint(
    content: String,
    languageVersionSettings: LanguageVersionSettings = FakeLanguageVersionSettings(),
    @Suppress("UNUSED_PARAMETER") compile: Boolean = true,
): List<Finding> {
    require(this !is RequiresAnalysisApi) {
        "${this.ruleName} requires Analysis API so you should use lintWithContext instead of lint"
    }
    return lint(compileContentForTest(content), languageVersionSettings)
}

fun Rule.lint(
    ktFile: KtFile,
    languageVersionSettings: LanguageVersionSettings = FakeLanguageVersionSettings(),
): List<Finding> {
    require(this !is RequiresAnalysisApi) {
        "${this.ruleName} requires Analysis Api so you should use lintWithContext instead of lint"
    }
    val findings = try {
        visitFile(ktFile, languageVersionSettings = languageVersionSettings).toList()
    } catch (@Suppress("TooGenericExceptionCaught") e: Throwable) {
        CaseRecorder.recordError(this, ktFile, e)
        throw e
    }
    CaseRecorder.record(this, ktFile, findings)
    return findings.filterSuppressed(this)
}

private fun List<Finding>.filterSuppressed(rule: Rule): List<Finding> =
    filterNot { it.entity.ktElement.isSuppressedBy(rule.ruleName) }

private fun KtElement.isSuppressedBy(id: RuleName): Boolean {
    if (id.value == "ForbiddenSuppress") return false

    fun KtElement.allAnnotationEntries(): Sequence<KtAnnotationEntry> {
        val element = this
        return sequence {
            if (element is KtAnnotated) {
                yieldAll(element.annotationEntries)
            }

            element.getStrictParentOfType<KtAnnotated>()?.let { yieldAll(it.allAnnotationEntries()) }
        }
    }

    return allAnnotationEntries()
        .filter { it.typeReference?.text == "Suppress" }
        .flatMap { it.valueArguments }
        .mapNotNull { it.getArgumentExpression()?.text }
        .map { it.replace("\"", "") }
        .any { it == id.value }
}
