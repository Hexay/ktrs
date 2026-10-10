package com.example.rules

import com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3
import com.pinterest.ktlint.rule.engine.core.api.AutocorrectDecision
import com.pinterest.ktlint.rule.engine.core.api.ElementType
import com.pinterest.ktlint.rule.engine.core.api.Rule
import com.pinterest.ktlint.rule.engine.core.api.RuleAutocorrectApproveHandler
import com.pinterest.ktlint.rule.engine.core.api.RuleId
import com.pinterest.ktlint.rule.engine.core.api.RuleProvider
import com.pinterest.ktlint.rule.engine.core.api.RuleSetId
import org.jetbrains.kotlin.com.intellij.lang.ASTNode

class NoVarRule :
    Rule(
        ruleId = RuleId("sample-rules:no-var"),
        about = About(maintainer = "ktrs", repositoryUrl = "https://github.com/Hexay/ktrs"),
    ),
    RuleAutocorrectApproveHandler {
    override fun beforeVisitChildNodes(
        node: ASTNode,
        emit: (offset: Int, errorMessage: String, canBeAutoCorrected: Boolean) -> AutocorrectDecision,
    ) {
        if (node.elementType == ElementType.VAR_KEYWORD) {
            emit(node.startOffset, "Use val instead of var", false)
        }
    }
}

class SampleRuleSetProvider : RuleSetProviderV3(RuleSetId("sample-rules")) {
    override fun getRuleProviders(): Set<RuleProvider> = setOf(RuleProvider { NoVarRule() })
}
