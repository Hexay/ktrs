import static org.jetbrains.kotlin.psi.psiUtil.PsiUtilsKt.getNextSiblingIgnoringWhitespace;
import static org.jetbrains.kotlin.psi.psiUtil.PsiUtilsKt.getNextSiblingIgnoringWhitespaceAndComments;
import static org.jetbrains.kotlin.psi.psiUtil.PsiUtilsKt.getPrevSiblingIgnoringWhitespace;
import static org.jetbrains.kotlin.psi.psiUtil.PsiUtilsKt.getPrevSiblingIgnoringWhitespaceAndComments;

import java.util.function.Supplier;

import org.jetbrains.kotlin.com.intellij.psi.PsiComment;
import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.com.intellij.psi.PsiErrorElement;
import org.jetbrains.kotlin.com.intellij.psi.PsiWhiteSpace;
import org.jetbrains.kotlin.com.intellij.psi.impl.source.tree.LeafPsiElement;
import org.jetbrains.kotlin.com.intellij.psi.util.PsiTreeUtil;
import org.jetbrains.kotlin.kdoc.psi.api.KDoc;
import org.jetbrains.kotlin.kdoc.psi.impl.KDocTag;
import org.jetbrains.kotlin.psi.*;

/** Per-element header: class, type tests, visitor dispatch, generic navigation. */
final class Base {
    private Base() {}

    private static final Class<?>[] TYPES = {
        PsiComment.class, PsiWhiteSpace.class, LeafPsiElement.class, PsiErrorElement.class,
        KtElement.class, KtExpression.class, KtDeclaration.class, KtNamedDeclaration.class,
        KtCallableDeclaration.class, KtDeclarationWithBody.class, KtFunction.class, KtClassOrObject.class,
        KtClass.class, KtModifierListOwner.class, KtTypeParameterListOwner.class, KtValVarKeywordOwner.class,
        KtDeclarationWithInitializer.class, KtConstructor.class, KtAnonymousInitializer.class,
        KtQualifiedExpression.class, KtReferenceExpression.class, KtSimpleNameExpression.class,
        KtCallElement.class, KtUnaryExpression.class, KtExpressionWithLabel.class, KtLoopExpression.class,
        KtWhileExpressionBase.class, KtDoubleColonExpression.class, KtContainerNode.class,
        KtContainerNodeForControlStructureBody.class, KtValueArgument.class, KtTypeElement.class,
        KtWhenCondition.class, KtStringTemplateEntry.class, KtStringTemplateEntryWithExpression.class,
        KtSuperTypeListEntry.class, KtModifierList.class, KtContextParameterList.class,
        KtContextReceiverList.class, KDoc.class, KDocTag.class,
    };

    static void line(StringBuilder sb, String key, Supplier<String> value) {
        sb.append(' ').append(key).append('=').append(Fmt.safe(value)).append('\n');
    }

    static void describe(PsiElement e, StringBuilder sb) {
        sb.append(Fmt.ref(e)).append(' ').append(e.getClass().getSimpleName()).append('\n');
        StringBuilder is = new StringBuilder();
        for (Class<?> type : TYPES) {
            if (!type.isInstance(e)) continue;
            if (is.length() > 0) is.append(',');
            is.append(type.getSimpleName());
        }
        sb.append(" is=").append(is).append('\n');
        Recorder recorder = new Recorder(false);
        e.accept(recorder);
        sb.append(" visit=").append(recorder.out).append('\n');

        line(sb, "children", () -> Fmt.refs(e.getChildren()));
        line(sb, "parent", () -> Fmt.ref(e.getParent()));
        line(sb, "firstChild", () -> Fmt.ref(e.getFirstChild()));
        line(sb, "lastChild", () -> Fmt.ref(e.getLastChild()));
        line(sb, "nextSibling", () -> Fmt.ref(e.getNextSibling()));
        line(sb, "prevSibling", () -> Fmt.ref(e.getPrevSibling()));
        line(sb, "nodeIsPsi", () -> String.valueOf(e.getNode() instanceof PsiElement));
        line(sb, "startsWithComment", () -> String.valueOf(e.getFirstChild() instanceof PsiComment));
        line(sb, "prevSiblingIgnoringWhitespace", () -> Fmt.ref(getPrevSiblingIgnoringWhitespace(e, false)));
        line(sb, "nextSiblingIgnoringWhitespace", () -> Fmt.ref(getNextSiblingIgnoringWhitespace(e, false)));
        line(sb, "prevSiblingIgnoringWhitespaceAndComments", () -> Fmt.ref(getPrevSiblingIgnoringWhitespaceAndComments(e, false)));
        line(sb, "prevSiblingIgnoringWhitespaceAndComments(withItself)", () -> Fmt.ref(getPrevSiblingIgnoringWhitespaceAndComments(e, true)));
        line(sb, "nextSiblingIgnoringWhitespaceAndComments", () -> Fmt.ref(getNextSiblingIgnoringWhitespaceAndComments(e, false)));
        line(sb, "prevLeaf", () -> Fmt.ref(PsiTreeUtil.prevLeaf(e, false)));
        line(sb, "parentOfType<KtStringTemplateExpression>", () -> Fmt.ref(PsiTreeUtil.getParentOfType(e, KtStringTemplateExpression.class, false)));

        Decls.describe(e, sb);
        Exprs.describe(e, sb);
        Members.describe(e, sb);
    }
}
