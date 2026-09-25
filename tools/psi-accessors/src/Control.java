import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.psi.*;

/** Accessors of when/if/loops/try and the binary-with-type operators that ktfmt calls. */
final class Control {
    private Control() {}

    static void describe(PsiElement e, StringBuilder sb) {
        if (e instanceof KtWhenExpression) {
            KtWhenExpression x = (KtWhenExpression) e;
            Base.line(sb, "KtWhenExpression.subjectExpression", () -> Fmt.ref(x.getSubjectExpression()));
            Base.line(sb, "KtWhenExpression.entries", () -> Fmt.refs(x.getEntries()));
        }
        if (e instanceof KtWhenEntry) {
            KtWhenEntry x = (KtWhenEntry) e;
            Base.line(sb, "KtWhenEntry.elseKeyword", () -> Fmt.ref(x.getElseKeyword()));
            Base.line(sb, "KtWhenEntry.conditions", () -> Fmt.refs(x.getConditions()));
            Base.line(sb, "KtWhenEntry.guard", () -> Fmt.ref(x.getGuard()));
            Base.line(sb, "KtWhenEntry.expression", () -> Fmt.ref(x.getExpression()));
            Base.line(sb, "KtWhenEntry.trailingComma", () -> Fmt.ref(x.getTrailingComma()));
        }
        if (e instanceof KtWhenEntryGuard) {
            KtWhenEntryGuard x = (KtWhenEntryGuard) e;
            Base.line(sb, "KtWhenEntryGuard.expression", () -> Fmt.ref(x.getExpression()));
        }
        if (e instanceof KtWhenConditionWithExpression) {
            KtWhenConditionWithExpression x = (KtWhenConditionWithExpression) e;
            Base.line(sb, "KtWhenConditionWithExpression.expression", () -> Fmt.ref(x.getExpression()));
        }
        if (e instanceof KtWhenConditionIsPattern) {
            KtWhenConditionIsPattern x = (KtWhenConditionIsPattern) e;
            Base.line(sb, "KtWhenConditionIsPattern.isNegated", () -> String.valueOf(x.isNegated()));
            Base.line(sb, "KtWhenConditionIsPattern.typeReference", () -> Fmt.ref(x.getTypeReference()));
        }
        if (e instanceof KtWhenConditionInRange) {
            KtWhenConditionInRange x = (KtWhenConditionInRange) e;
            Base.line(sb, "KtWhenConditionInRange.rangeExpression", () -> Fmt.ref(x.getRangeExpression()));
        }
        if (e instanceof KtIfExpression) {
            KtIfExpression x = (KtIfExpression) e;
            Base.line(sb, "KtIfExpression.condition", () -> Fmt.ref(x.getCondition()));
            Base.line(sb, "KtIfExpression.then", () -> Fmt.ref(x.getThen()));
            Base.line(sb, "KtIfExpression.elseKeyword", () -> Fmt.ref(x.getElseKeyword()));
            Base.line(sb, "KtIfExpression.else", () -> Fmt.ref(x.getElse()));
        }
        if (e instanceof KtLoopExpression) {
            KtLoopExpression x = (KtLoopExpression) e;
            Base.line(sb, "KtLoopExpression.body", () -> Fmt.ref(x.getBody()));
        }
        if (e instanceof KtForExpression) {
            KtForExpression x = (KtForExpression) e;
            Base.line(sb, "KtForExpression.loopParameter", () -> Fmt.ref(x.getLoopParameter()));
            Base.line(sb, "KtForExpression.loopRange", () -> Fmt.ref(x.getLoopRange()));
        }
        if (e instanceof KtWhileExpressionBase) {
            KtWhileExpressionBase x = (KtWhileExpressionBase) e;
            Base.line(sb, "KtWhileExpressionBase.condition", () -> Fmt.ref(x.getCondition()));
        }
        if (e instanceof KtIsExpression) {
            KtIsExpression x = (KtIsExpression) e;
            Base.line(sb, "KtIsExpression.leftHandSide", () -> Fmt.ref(x.getLeftHandSide()));
            Base.line(sb, "KtIsExpression.operationReference", () -> Fmt.ref(x.getOperationReference()));
            Base.line(sb, "KtIsExpression.typeReference", () -> Fmt.ref(x.getTypeReference()));
        }
        if (e instanceof KtBinaryExpressionWithTypeRHS) {
            KtBinaryExpressionWithTypeRHS x = (KtBinaryExpressionWithTypeRHS) e;
            Base.line(sb, "KtBinaryExpressionWithTypeRHS.left", () -> Fmt.ref(x.getLeft()));
            Base.line(sb, "KtBinaryExpressionWithTypeRHS.operationReference", () -> Fmt.ref(x.getOperationReference()));
            Base.line(sb, "KtBinaryExpressionWithTypeRHS.right", () -> Fmt.ref(x.getRight()));
        }
        if (e instanceof KtTryExpression) {
            KtTryExpression x = (KtTryExpression) e;
            Base.line(sb, "KtTryExpression.tryBlock", () -> Fmt.ref(x.getTryBlock()));
            Base.line(sb, "KtTryExpression.catchClauses", () -> Fmt.refs(x.getCatchClauses()));
            Base.line(sb, "KtTryExpression.finallyBlock", () -> Fmt.ref(x.getFinallyBlock()));
        }
        if (e instanceof KtCatchClause) {
            KtCatchClause x = (KtCatchClause) e;
            Base.line(sb, "KtCatchClause.catchParameter", () -> Fmt.ref(x.getCatchParameter()));
            Base.line(sb, "KtCatchClause.catchBody", () -> Fmt.ref(x.getCatchBody()));
        }
        if (e instanceof KtFinallySection) {
            KtFinallySection x = (KtFinallySection) e;
            Base.line(sb, "KtFinallySection.finalExpression", () -> Fmt.ref(x.getFinalExpression()));
        }
        if (e instanceof KtThrowExpression) {
            KtThrowExpression x = (KtThrowExpression) e;
            Base.line(sb, "KtThrowExpression.thrownExpression", () -> Fmt.ref(x.getThrownExpression()));
        }
    }
}
