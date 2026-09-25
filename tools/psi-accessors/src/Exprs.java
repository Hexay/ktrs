import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.com.intellij.psi.util.PsiTreeUtil;
import org.jetbrains.kotlin.kdoc.psi.impl.KDocImpl;
import org.jetbrains.kotlin.kdoc.psi.impl.KDocLink;
import org.jetbrains.kotlin.kdoc.psi.impl.KDocName;
import org.jetbrains.kotlin.kdoc.psi.impl.KDocSection;
import org.jetbrains.kotlin.kdoc.psi.impl.KDocTag;
import org.jetbrains.kotlin.psi.*;

/** Accessors of expressions, control flow, arguments and KDoc that ktfmt calls. */
final class Exprs {
    private Exprs() {}

    static void describe(PsiElement e, StringBuilder sb) {
        if (e instanceof KtQualifiedExpression) {
            KtQualifiedExpression x = (KtQualifiedExpression) e;
            Base.line(sb, "KtQualifiedExpression.receiverExpression", () -> Fmt.ref(x.getReceiverExpression()));
            Base.line(sb, "KtQualifiedExpression.selectorExpression", () -> Fmt.ref(x.getSelectorExpression()));
            Base.line(sb, "KtQualifiedExpression.operationSign", () -> Fmt.kind(x.getOperationSign()));
        }
        if (e instanceof KtArrayAccessExpression) {
            KtArrayAccessExpression x = (KtArrayAccessExpression) e;
            Base.line(sb, "KtArrayAccessExpression.arrayExpression", () -> Fmt.ref(x.getArrayExpression()));
            Base.line(sb, "KtArrayAccessExpression.indexExpressions", () -> Fmt.refs(x.getIndexExpressions()));
            Base.line(sb, "KtArrayAccessExpression.trailingComma", () -> Fmt.ref(x.getTrailingComma()));
        }
        if (e instanceof KtUnaryExpression) {
            KtUnaryExpression x = (KtUnaryExpression) e;
            Base.line(sb, "KtUnaryExpression.baseExpression", () -> Fmt.ref(x.getBaseExpression()));
            Base.line(sb, "KtUnaryExpression.operationReference", () -> Fmt.ref(x.getOperationReference()));
        }
        if (e instanceof KtBinaryExpression) {
            KtBinaryExpression x = (KtBinaryExpression) e;
            Base.line(sb, "KtBinaryExpression.left", () -> Fmt.ref(x.getLeft()));
            Base.line(sb, "KtBinaryExpression.right", () -> Fmt.ref(x.getRight()));
            Base.line(sb, "KtBinaryExpression.operationReference", () -> Fmt.ref(x.getOperationReference()));
            Base.line(sb, "KtBinaryExpression.operationToken", () -> Fmt.kind(x.getOperationToken()));
        }
        if (e instanceof KtValueArgumentList) {
            KtValueArgumentList x = (KtValueArgumentList) e;
            Base.line(sb, "KtValueArgumentList.arguments", () -> Fmt.refs(x.getArguments()));
            Base.line(sb, "KtValueArgumentList.trailingComma", () -> Fmt.ref(x.getTrailingComma()));
            Base.line(sb, "KtValueArgumentList.leftParenthesis", () -> Fmt.ref(x.getLeftParenthesis()));
            Base.line(sb, "KtValueArgumentList.rightParenthesis", () -> Fmt.ref(x.getRightParenthesis()));
        }
        if (e instanceof KtValueArgument) {
            KtValueArgument x = (KtValueArgument) e;
            Base.line(sb, "KtValueArgument.argumentExpression", () -> Fmt.ref(x.getArgumentExpression()));
            Base.line(sb, "KtValueArgument.argumentName", () -> Fmt.ref(x.getArgumentName()));
            Base.line(sb, "KtValueArgument.isSpread", () -> String.valueOf(x.isSpread()));
            Base.line(sb, "KtValueArgument.textOffset", () -> String.valueOf(x.getTextOffset()));
        }
        if (e instanceof KtLambdaExpression) {
            KtLambdaExpression x = (KtLambdaExpression) e;
            Base.line(sb, "KtLambdaExpression.functionLiteral", () -> Fmt.ref(x.getFunctionLiteral()));
            Base.line(sb, "KtLambdaExpression.valueParameters", () -> Fmt.refs(x.getValueParameters()));
            Base.line(sb, "KtLambdaExpression.bodyExpression", () -> Fmt.ref(x.getBodyExpression()));
        }
        if (e instanceof KtFunctionLiteral) {
            KtFunctionLiteral x = (KtFunctionLiteral) e;
            Base.line(sb, "KtFunctionLiteral.arrow", () -> Fmt.ref(x.getArrow()));
        }
        if (e instanceof KtExpressionWithLabel) {
            KtExpressionWithLabel x = (KtExpressionWithLabel) e;
            Base.line(sb, "KtExpressionWithLabel.targetLabel", () -> Fmt.ref(x.getTargetLabel()));
            Base.line(sb, "KtExpressionWithLabel.labelQualifier", () -> Fmt.ref(x.getLabelQualifier()));
        }
        if (e instanceof KtReturnExpression) {
            KtReturnExpression x = (KtReturnExpression) e;
            Base.line(sb, "KtReturnExpression.returnedExpression", () -> Fmt.ref(x.getReturnedExpression()));
        }
        if (e instanceof KtLabeledExpression) {
            KtLabeledExpression x = (KtLabeledExpression) e;
            Base.line(sb, "KtLabeledExpression.baseExpression", () -> Fmt.ref(x.getBaseExpression()));
        }
        if (e instanceof KtSimpleNameExpression) {
            KtSimpleNameExpression x = (KtSimpleNameExpression) e;
            Base.line(sb, "KtSimpleNameExpression.identifier", () -> Fmt.ref(x.getIdentifier()));
            Base.line(sb, "KtSimpleNameExpression.referencedName", () -> Fmt.str(x.getReferencedName()));
            Base.line(sb, "KtSimpleNameExpression.referencedNameElementType", () -> Fmt.kind(x.getReferencedNameElementType()));
        }
        if (e instanceof KtSuperExpression) {
            KtSuperExpression x = (KtSuperExpression) e;
            Base.line(sb, "KtSuperExpression.superTypeQualifier", () -> Fmt.ref(x.getSuperTypeQualifier()));
        }
        if (e instanceof KtPropertyDelegate) {
            KtPropertyDelegate x = (KtPropertyDelegate) e;
            Base.line(sb, "KtPropertyDelegate.expression", () -> Fmt.ref(x.getExpression()));
        }
        if (e instanceof KtParenthesizedExpression) {
            KtParenthesizedExpression x = (KtParenthesizedExpression) e;
            Base.line(sb, "KtParenthesizedExpression.expression", () -> Fmt.ref(x.getExpression()));
        }
        if (e instanceof KtAnnotatedExpression) {
            KtAnnotatedExpression x = (KtAnnotatedExpression) e;
            Base.line(sb, "KtAnnotatedExpression.baseExpression", () -> Fmt.ref(x.getBaseExpression()));
            Base.line(sb, "KtAnnotatedExpression.annotationEntries", () -> Fmt.refs(x.getAnnotationEntries()));
        }
        Control.describe(e, sb);
        if (e instanceof KtDoubleColonExpression) {
            KtDoubleColonExpression x = (KtDoubleColonExpression) e;
            Base.line(sb, "KtDoubleColonExpression.receiverExpression", () -> Fmt.ref(x.getReceiverExpression()));
            Base.line(sb, "KtDoubleColonExpression.hasQuestionMarks", () -> String.valueOf(x.getHasQuestionMarks()));
        }
        if (e instanceof KtCallableReferenceExpression) {
            KtCallableReferenceExpression x = (KtCallableReferenceExpression) e;
            Base.line(sb, "KtCallableReferenceExpression.callableReference", () -> Fmt.ref(x.getCallableReference()));
        }
        if (e instanceof KtCollectionLiteralExpression) {
            KtCollectionLiteralExpression x = (KtCollectionLiteralExpression) e;
            Base.line(sb, "KtCollectionLiteralExpression.innerExpressions", () -> Fmt.refs(x.getInnerExpressions()));
            Base.line(sb, "KtCollectionLiteralExpression.trailingComma", () -> Fmt.ref(x.getTrailingComma()));
        }
        if (e instanceof KDocImpl) {
            Base.line(sb, "KDocImpl.childrenOfType<KDocSection>", () -> childrenOfType(e, KDocSection.class));
        }
        if (e instanceof KDocTag) {
            Base.line(sb, "KDocTag.childrenOfType<KDocTag>", () -> childrenOfType(e, KDocTag.class));
            Base.line(sb, "KDocTag.childrenOfType<KDocLink>", () -> childrenOfType(e, KDocLink.class));
        }
        if (e instanceof KDocLink) {
            Base.line(sb, "KDocLink.childrenOfType<KDocName>", () -> childrenOfType(e, KDocName.class));
        }
        if (e instanceof KDocName) {
            KDocName x = (KDocName) e;
            Base.line(sb, "KDocName.qualifiedName", () -> Fmt.strs(x.getQualifiedName()));
        }
    }

    /** psiUtil `getChildrenOfType<T>()`: PsiTreeUtil's null for "none" becomes an empty array. */
    private static String childrenOfType(PsiElement e, Class<? extends PsiElement> type) {
        PsiElement[] result = PsiTreeUtil.getChildrenOfType(e, type);
        return Fmt.refs(result == null ? new PsiElement[0] : result);
    }
}
