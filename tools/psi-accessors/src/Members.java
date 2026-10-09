import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

import org.jetbrains.kotlin.com.intellij.openapi.util.TextRange;
import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.com.intellij.psi.util.PsiTreeUtil;
import org.jetbrains.kotlin.psi.*;
import org.jetbrains.kotlin.psi.psiUtil.KtPsiUtilKt;
import org.jetbrains.kotlin.psi.psiUtil.PsiUtilsKt;

/** Accessors and psiUtil helpers the detekt port (crates/ktrs-detekt) calls beyond ktfmt's. */
final class Members {
    private Members() {}

    static void describe(PsiElement e, StringBuilder sb) {
        Base.line(sb, "textOffset", () -> String.valueOf(e.getTextOffset()));
        Base.line(sb, "startOffsetSkippingComments", () -> String.valueOf(PsiUtilsKt.getStartOffsetSkippingComments(e)));
        Base.line(sb, "findDescendantOfType<KtProperty>", () -> Fmt.ref(PsiTreeUtil.findChildOfType(e, KtProperty.class, false)));
        Base.line(sb, "findChildrenOfType<KtNamedFunction>", () -> {
            // The collection's order is not part of the contract: by offset.
            List<KtNamedFunction> functions = new ArrayList<>(PsiTreeUtil.findChildrenOfType(e, KtNamedFunction.class));
            functions.sort(Comparator.comparingInt(f -> f.getTextRange().getStartOffset()));
            return Fmt.refs(functions);
        });
        if (e instanceof KtAnnotated) {
            KtAnnotated x = (KtAnnotated) e;
            Base.line(sb, "KtAnnotated.annotationEntries", () -> Fmt.refs(x.getAnnotationEntries()));
        }
        if (e instanceof KtModifierListOwner) {
            KtModifierListOwner x = (KtModifierListOwner) e;
            Base.line(sb, "KtModifierListOwner.visibilityModifierType", () -> Fmt.kind(KtPsiUtilKt.visibilityModifierType(x)));
            Base.line(sb, "KtModifierListOwner.isPrivate", () -> String.valueOf(KtPsiUtilKt.isPrivate(x)));
            Base.line(sb, "KtModifierListOwner.hasExpectModifier", () -> String.valueOf(PsiUtilsKt.hasExpectModifier(x)));
        }
        // KtScript's name comes from the file name, which ktrs-psi's tree does not have.
        if (e instanceof KtNamedDeclaration && !(e instanceof KtScript)) {
            KtNamedDeclaration x = (KtNamedDeclaration) e;
            Base.line(sb, "KtNamedDeclaration.name", () -> Fmt.str(x.getName()));
            Base.line(sb, "KtNamedDeclaration.nameAsSafeName", () -> Fmt.str(x.getNameAsSafeName().asString()) + " " + x.getNameAsSafeName().isSpecial());
        }
        if (e instanceof KtTypeParameterListOwner) {
            KtTypeParameterListOwner x = (KtTypeParameterListOwner) e;
            Base.line(sb, "KtTypeParameterListOwner.typeParameters", () -> Fmt.refs(x.getTypeParameters()));
        }
        if (e instanceof KtDeclaration) {
            KtDeclaration x = (KtDeclaration) e;
            Base.line(sb, "KtDeclaration.containingClassOrObject", () -> Fmt.ref(KtPsiUtilKt.getContainingClassOrObject(x)));
            Base.line(sb, "KtDeclaration.containingClass", () -> Fmt.ref(KtPsiUtilKt.containingClass(x)));
        }
        if (e instanceof KtClassOrObject) {
            KtClassOrObject x = (KtClassOrObject) e;
            Base.line(sb, "KtClassOrObject.declarations", () -> Fmt.refs(x.getDeclarations()));
            Base.line(sb, "KtClassOrObject.secondaryConstructors", () -> Fmt.refs(x.getSecondaryConstructors()));
            Base.line(sb, "KtClassOrObject.superTypeListEntries", () -> Fmt.refs(x.getSuperTypeListEntries()));
            Base.line(sb, "KtClassOrObject.isTopLevel", () -> String.valueOf(x.isTopLevel()));
            Base.line(sb, "KtClassOrObject.isObjectLiteral", () -> String.valueOf(KtPsiUtilKt.isObjectLiteral(x)));
        }
        if (e instanceof KtClass) {
            KtClass x = (KtClass) e;
            Base.line(sb, "KtClass.isInterface", () -> String.valueOf(x.isInterface()));
        }
        if (e instanceof KtFile) {
            KtFile x = (KtFile) e;
            Base.line(sb, "KtFile.declarations", () -> Fmt.refs(x.getDeclarations()));
            Base.line(sb, "KtFile.packageDirective", () -> Fmt.ref(x.getPackageDirective()));
            Base.line(sb, "KtFile.packageFqName", () -> Fmt.str(x.getPackageFqName().asString()));
            lines(x, sb);
        }
        if (e instanceof KtNamedFunction) {
            KtNamedFunction x = (KtNamedFunction) e;
            Base.line(sb, "KtNamedFunction.isTopLevel", () -> String.valueOf(x.isTopLevel()));
            Base.line(sb, "KtNamedFunction.hasBody", () -> String.valueOf(x.hasBody()));
            Base.line(sb, "KtNamedFunction.isLocal", () -> String.valueOf(x.isLocal()));
        }
        if (e instanceof KtProperty) {
            KtProperty x = (KtProperty) e;
            Base.line(sb, "KtProperty.isTopLevel", () -> String.valueOf(x.isTopLevel()));
            Base.line(sb, "KtProperty.isMember", () -> String.valueOf(x.isMember()));
            Base.line(sb, "KtProperty.isLocal", () -> String.valueOf(x.isLocal()));
        }
        if (e instanceof KtParameter) {
            KtParameter x = (KtParameter) e;
            Base.line(sb, "KtParameter.ownerFunction", () -> Fmt.ref(x.getOwnerFunction()));
        }
        if (e instanceof KtBlockExpression) {
            KtBlockExpression x = (KtBlockExpression) e;
            Base.line(sb, "KtBlockExpression.statements", () -> Fmt.refs(x.getStatements()));
        }
        if (e instanceof KtReturnExpression) {
            KtReturnExpression x = (KtReturnExpression) e;
            Base.line(sb, "KtReturnExpression.labeledExpression", () -> Fmt.ref(x.getLabeledExpression()));
        }
        if (e instanceof KtLambdaArgument) {
            KtLambdaArgument x = (KtLambdaArgument) e;
            Base.line(sb, "KtLambdaArgument.lambdaExpression", () -> Fmt.ref(x.getLambdaExpression()));
        }
        if (e instanceof KtCallElement) {
            KtCallElement x = (KtCallElement) e;
            Base.line(sb, "KtCallElement.callNameExpression", () -> Fmt.ref(KtPsiUtilKt.getCallNameExpression(x)));
            Base.line(sb, "KtCallElement.valueArguments", () -> {
                List<PsiElement> arguments = new ArrayList<>();
                x.getValueArguments().forEach(argument -> arguments.add(argument.asElement()));
                return Fmt.refs(arguments);
            });
        }
        if (e instanceof KtExpression) {
            KtExpression x = (KtExpression) e;
            Base.line(sb, "KtExpression.qualifiedExpressionForReceiverOrThis", () -> Fmt.ref(KtPsiUtilKt.getQualifiedExpressionForReceiverOrThis(x)));
            Base.line(sb, "KtExpression.lastBlockStatementOrThis", () -> Fmt.ref(KtPsiUtilKt.lastBlockStatementOrThis(x)));
        }
    }

    /** Per line of the file: `findElementAt(lineStart)` and `elementsInRange(line)`. */
    private static void lines(KtFile file, StringBuilder sb) {
        String text = file.getText();
        int start = 0;
        while (start <= text.length()) {
            int newline = text.indexOf('\n', start);
            int end = newline < 0 ? text.length() : newline;
            int lineStart = start;
            Base.line(sb, "KtFile.findElementAt(" + lineStart + ")", () -> Fmt.ref(file.findElementAt(lineStart)));
            Base.line(sb, "KtFile.elementsInRange(" + lineStart + ".." + end + ")", () -> Fmt.refs(PsiUtilsKt.elementsInRange(file, new TextRange(lineStart, end))));
            if (newline < 0) break;
            start = newline + 1;
        }
    }
}
