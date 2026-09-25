import org.jetbrains.kotlin.com.intellij.psi.PsiComment;
import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.com.intellij.psi.PsiErrorElement;
import org.jetbrains.kotlin.com.intellij.psi.PsiFile;
import org.jetbrains.kotlin.com.intellij.psi.PsiWhiteSpace;
import org.jetbrains.kotlin.psi.*;

/**
 * Records every visit method a KtVisitorVoid receives. With tree=false, one accept() yields the dispatch chain
 * (visitDotQualifiedExpression, visitQualifiedExpression, ..., visitElement); with tree=true it behaves like
 * KtTreeVisitorVoid and records the whole walk.
 */
final class Recorder extends KtVisitorVoid {
    private final boolean tree;
    final StringBuilder out = new StringBuilder();

    Recorder(boolean tree) {
        this.tree = tree;
    }

    private void r(String name, PsiElement e) {
        if (tree) {
            out.append(' ').append(name).append(' ').append(Fmt.ref(e)).append('\n');
        } else {
            if (out.length() > 0) out.append(',');
            out.append(name);
        }
    }

    @Override public void visitElement(PsiElement e) { r("visitElement", e); if (tree) e.acceptChildren(this); }
    @Override public void visitFile(PsiFile e) { r("visitFile", e); super.visitFile(e); }
    @Override public void visitComment(PsiComment e) { r("visitComment", e); super.visitComment(e); }
    @Override public void visitWhiteSpace(PsiWhiteSpace e) { r("visitWhiteSpace", e); super.visitWhiteSpace(e); }
    @Override public void visitErrorElement(PsiErrorElement e) { r("visitErrorElement", e); super.visitErrorElement(e); }

    @Override public void visitKtElement(KtElement e) { r("visitKtElement", e); super.visitKtElement(e); }
    @Override public void visitDeclaration(KtDeclaration e) { r("visitDeclaration", e); super.visitDeclaration(e); }
    @Override public void visitClass(KtClass e) { r("visitClass", e); super.visitClass(e); }
    @Override public void visitClassOrObject(KtClassOrObject e) { r("visitClassOrObject", e); super.visitClassOrObject(e); }
    @Override public void visitConstructor(KtConstructor<?> e) { r("visitConstructor", e); super.visitConstructor(e); }
    @Override public void visitSecondaryConstructor(KtSecondaryConstructor e) { r("visitSecondaryConstructor", e); super.visitSecondaryConstructor(e); }
    @Override public void visitPrimaryConstructor(KtPrimaryConstructor e) { r("visitPrimaryConstructor", e); super.visitPrimaryConstructor(e); }
    @Override public void visitNamedFunction(KtNamedFunction e) { r("visitNamedFunction", e); super.visitNamedFunction(e); }
    @Override public void visitProperty(KtProperty e) { r("visitProperty", e); super.visitProperty(e); }
    @Override public void visitTypeAlias(KtTypeAlias e) { r("visitTypeAlias", e); super.visitTypeAlias(e); }
    @Override public void visitDestructuringDeclaration(KtDestructuringDeclaration e) { r("visitDestructuringDeclaration", e); super.visitDestructuringDeclaration(e); }
    @Override public void visitDestructuringDeclarationEntry(KtDestructuringDeclarationEntry e) { r("visitDestructuringDeclarationEntry", e); super.visitDestructuringDeclarationEntry(e); }
    @Override public void visitKtFile(KtFile e) { r("visitKtFile", e); super.visitKtFile(e); }
    @Override public void visitScript(KtScript e) { r("visitScript", e); super.visitScript(e); }
    @Override public void visitImportAlias(KtImportAlias e) { r("visitImportAlias", e); super.visitImportAlias(e); }
    @Override public void visitImportDirective(KtImportDirective e) { r("visitImportDirective", e); super.visitImportDirective(e); }
    @Override public void visitImportList(KtImportList e) { r("visitImportList", e); super.visitImportList(e); }
    @Override public void visitClassBody(KtClassBody e) { r("visitClassBody", e); super.visitClassBody(e); }
    @Override public void visitCompanionBlock(KtCompanionBlock e) { r("visitCompanionBlock", e); super.visitCompanionBlock(e); }
    @Override public void visitModifierList(KtModifierList e) { r("visitModifierList", e); super.visitModifierList(e); }
    @Override public void visitAnnotation(KtAnnotation e) { r("visitAnnotation", e); super.visitAnnotation(e); }
    @Override public void visitAnnotationEntry(KtAnnotationEntry e) { r("visitAnnotationEntry", e); super.visitAnnotationEntry(e); }
    @Override public void visitConstructorCalleeExpression(KtConstructorCalleeExpression e) { r("visitConstructorCalleeExpression", e); super.visitConstructorCalleeExpression(e); }
    @Override public void visitTypeParameterList(KtTypeParameterList e) { r("visitTypeParameterList", e); super.visitTypeParameterList(e); }
    @Override public void visitTypeParameter(KtTypeParameter e) { r("visitTypeParameter", e); super.visitTypeParameter(e); }
    @Override public void visitEnumEntry(KtEnumEntry e) { r("visitEnumEntry", e); super.visitEnumEntry(e); }
    @Override public void visitParameterList(KtParameterList e) { r("visitParameterList", e); super.visitParameterList(e); }
    @Override public void visitParameter(KtParameter e) { r("visitParameter", e); super.visitParameter(e); }
    @Override public void visitSuperTypeList(KtSuperTypeList e) { r("visitSuperTypeList", e); super.visitSuperTypeList(e); }
    @Override public void visitSuperTypeListEntry(KtSuperTypeListEntry e) { r("visitSuperTypeListEntry", e); super.visitSuperTypeListEntry(e); }
    @Override public void visitDelegatedSuperTypeEntry(KtDelegatedSuperTypeEntry e) { r("visitDelegatedSuperTypeEntry", e); super.visitDelegatedSuperTypeEntry(e); }
    @Override public void visitSuperTypeCallEntry(KtSuperTypeCallEntry e) { r("visitSuperTypeCallEntry", e); super.visitSuperTypeCallEntry(e); }
    @Override public void visitSuperTypeEntry(KtSuperTypeEntry e) { r("visitSuperTypeEntry", e); super.visitSuperTypeEntry(e); }
    @Override public void visitContextReceiverList(KtContextReceiverList e) { r("visitContextReceiverList", e); super.visitContextReceiverList(e); }
    @Override public void visitContextParameterList(KtContextParameterList e) { r("visitContextParameterList", e); super.visitContextParameterList(e); }
    @Override public void visitContextReceiver(KtContextReceiver e) { r("visitContextReceiver", e); super.visitContextReceiver(e); }
    @Override public void visitConstructorDelegationCall(KtConstructorDelegationCall e) { r("visitConstructorDelegationCall", e); super.visitConstructorDelegationCall(e); }
    @Override public void visitPropertyDelegate(KtPropertyDelegate e) { r("visitPropertyDelegate", e); super.visitPropertyDelegate(e); }
    @Override public void visitTypeReference(KtTypeReference e) { r("visitTypeReference", e); super.visitTypeReference(e); }
    @Override public void visitValueArgumentList(KtValueArgumentList e) { r("visitValueArgumentList", e); super.visitValueArgumentList(e); }
    @Override public void visitArgument(KtValueArgument e) { r("visitArgument", e); super.visitArgument(e); }
    @Override public void visitExpression(KtExpression e) { r("visitExpression", e); super.visitExpression(e); }
    @Override public void visitLoopExpression(KtLoopExpression e) { r("visitLoopExpression", e); super.visitLoopExpression(e); }
    @Override public void visitConstantExpression(KtConstantExpression e) { r("visitConstantExpression", e); super.visitConstantExpression(e); }
    @Override public void visitSimpleNameExpression(KtSimpleNameExpression e) { r("visitSimpleNameExpression", e); super.visitSimpleNameExpression(e); }
    @Override public void visitReferenceExpression(KtReferenceExpression e) { r("visitReferenceExpression", e); super.visitReferenceExpression(e); }
    @Override public void visitLabeledExpression(KtLabeledExpression e) { r("visitLabeledExpression", e); super.visitLabeledExpression(e); }
    @Override public void visitPrefixExpression(KtPrefixExpression e) { r("visitPrefixExpression", e); super.visitPrefixExpression(e); }
    @Override public void visitPostfixExpression(KtPostfixExpression e) { r("visitPostfixExpression", e); super.visitPostfixExpression(e); }
    @Override public void visitUnaryExpression(KtUnaryExpression e) { r("visitUnaryExpression", e); super.visitUnaryExpression(e); }
    @Override public void visitBinaryExpression(KtBinaryExpression e) { r("visitBinaryExpression", e); super.visitBinaryExpression(e); }
    @Override public void visitReturnExpression(KtReturnExpression e) { r("visitReturnExpression", e); super.visitReturnExpression(e); }
    @Override public void visitExpressionWithLabel(KtExpressionWithLabel e) { r("visitExpressionWithLabel", e); super.visitExpressionWithLabel(e); }
    @Override public void visitThrowExpression(KtThrowExpression e) { r("visitThrowExpression", e); super.visitThrowExpression(e); }
    @Override public void visitBreakExpression(KtBreakExpression e) { r("visitBreakExpression", e); super.visitBreakExpression(e); }
    @Override public void visitContinueExpression(KtContinueExpression e) { r("visitContinueExpression", e); super.visitContinueExpression(e); }
    @Override public void visitIfExpression(KtIfExpression e) { r("visitIfExpression", e); super.visitIfExpression(e); }
    @Override public void visitWhenExpression(KtWhenExpression e) { r("visitWhenExpression", e); super.visitWhenExpression(e); }
    @Override public void visitCollectionLiteralExpression(KtCollectionLiteralExpression e) { r("visitCollectionLiteralExpression", e); super.visitCollectionLiteralExpression(e); }
    @Override public void visitTryExpression(KtTryExpression e) { r("visitTryExpression", e); super.visitTryExpression(e); }
    @Override public void visitForExpression(KtForExpression e) { r("visitForExpression", e); super.visitForExpression(e); }
    @Override public void visitWhileExpression(KtWhileExpression e) { r("visitWhileExpression", e); super.visitWhileExpression(e); }
    @Override public void visitDoWhileExpression(KtDoWhileExpression e) { r("visitDoWhileExpression", e); super.visitDoWhileExpression(e); }
    @Override public void visitLambdaExpression(KtLambdaExpression e) { r("visitLambdaExpression", e); super.visitLambdaExpression(e); }
    @Override public void visitAnnotatedExpression(KtAnnotatedExpression e) { r("visitAnnotatedExpression", e); super.visitAnnotatedExpression(e); }
    @Override public void visitCallExpression(KtCallExpression e) { r("visitCallExpression", e); super.visitCallExpression(e); }
    @Override public void visitArrayAccessExpression(KtArrayAccessExpression e) { r("visitArrayAccessExpression", e); super.visitArrayAccessExpression(e); }
    @Override public void visitQualifiedExpression(KtQualifiedExpression e) { r("visitQualifiedExpression", e); super.visitQualifiedExpression(e); }
    @Override public void visitDoubleColonExpression(KtDoubleColonExpression e) { r("visitDoubleColonExpression", e); super.visitDoubleColonExpression(e); }
    @Override public void visitCallableReferenceExpression(KtCallableReferenceExpression e) { r("visitCallableReferenceExpression", e); super.visitCallableReferenceExpression(e); }
    @Override public void visitClassLiteralExpression(KtClassLiteralExpression e) { r("visitClassLiteralExpression", e); super.visitClassLiteralExpression(e); }
    @Override public void visitDotQualifiedExpression(KtDotQualifiedExpression e) { r("visitDotQualifiedExpression", e); super.visitDotQualifiedExpression(e); }
    @Override public void visitSafeQualifiedExpression(KtSafeQualifiedExpression e) { r("visitSafeQualifiedExpression", e); super.visitSafeQualifiedExpression(e); }
    @Override public void visitObjectLiteralExpression(KtObjectLiteralExpression e) { r("visitObjectLiteralExpression", e); super.visitObjectLiteralExpression(e); }
    @Override public void visitBlockExpression(KtBlockExpression e) { r("visitBlockExpression", e); super.visitBlockExpression(e); }
    @Override public void visitCatchSection(KtCatchClause e) { r("visitCatchSection", e); super.visitCatchSection(e); }
    @Override public void visitFinallySection(KtFinallySection e) { r("visitFinallySection", e); super.visitFinallySection(e); }
    @Override public void visitTypeArgumentList(KtTypeArgumentList e) { r("visitTypeArgumentList", e); super.visitTypeArgumentList(e); }
    @Override public void visitThisExpression(KtThisExpression e) { r("visitThisExpression", e); super.visitThisExpression(e); }
    @Override public void visitSuperExpression(KtSuperExpression e) { r("visitSuperExpression", e); super.visitSuperExpression(e); }
    @Override public void visitParenthesizedExpression(KtParenthesizedExpression e) { r("visitParenthesizedExpression", e); super.visitParenthesizedExpression(e); }
    @Override public void visitInitializerList(KtInitializerList e) { r("visitInitializerList", e); super.visitInitializerList(e); }
    @Override public void visitAnonymousInitializer(KtAnonymousInitializer e) { r("visitAnonymousInitializer", e); super.visitAnonymousInitializer(e); }
    @Override public void visitScriptInitializer(KtScriptInitializer e) { r("visitScriptInitializer", e); super.visitScriptInitializer(e); }
    @Override public void visitClassInitializer(KtClassInitializer e) { r("visitClassInitializer", e); super.visitClassInitializer(e); }
    @Override public void visitPropertyAccessor(KtPropertyAccessor e) { r("visitPropertyAccessor", e); super.visitPropertyAccessor(e); }
    @Override public void visitTypeConstraintList(KtTypeConstraintList e) { r("visitTypeConstraintList", e); super.visitTypeConstraintList(e); }
    @Override public void visitTypeConstraint(KtTypeConstraint e) { r("visitTypeConstraint", e); super.visitTypeConstraint(e); }
    @Override public void visitUserType(KtUserType e) { r("visitUserType", e); super.visitUserType(e); }
    @Override public void visitDynamicType(KtDynamicType e) { r("visitDynamicType", e); super.visitDynamicType(e); }
    @Override public void visitFunctionType(KtFunctionType e) { r("visitFunctionType", e); super.visitFunctionType(e); }
    @Override public void visitSelfType(KtSelfType e) { r("visitSelfType", e); super.visitSelfType(e); }
    @Override public void visitBinaryWithTypeRHSExpression(KtBinaryExpressionWithTypeRHS e) { r("visitBinaryWithTypeRHSExpression", e); super.visitBinaryWithTypeRHSExpression(e); }
    @Override public void visitStringTemplateExpression(KtStringTemplateExpression e) { r("visitStringTemplateExpression", e); super.visitStringTemplateExpression(e); }
    @Override public void visitNamedDeclaration(KtNamedDeclaration e) { r("visitNamedDeclaration", e); super.visitNamedDeclaration(e); }
    @Override public void visitNullableType(KtNullableType e) { r("visitNullableType", e); super.visitNullableType(e); }
    @Override public void visitIntersectionType(KtIntersectionType e) { r("visitIntersectionType", e); super.visitIntersectionType(e); }
    @Override public void visitTypeProjection(KtTypeProjection e) { r("visitTypeProjection", e); super.visitTypeProjection(e); }
    @Override public void visitWhenEntry(KtWhenEntry e) { r("visitWhenEntry", e); super.visitWhenEntry(e); }
    @Override public void visitIsExpression(KtIsExpression e) { r("visitIsExpression", e); super.visitIsExpression(e); }
    @Override public void visitWhenConditionIsPattern(KtWhenConditionIsPattern e) { r("visitWhenConditionIsPattern", e); super.visitWhenConditionIsPattern(e); }
    @Override public void visitWhenConditionInRange(KtWhenConditionInRange e) { r("visitWhenConditionInRange", e); super.visitWhenConditionInRange(e); }
    @Override public void visitWhenConditionWithExpression(KtWhenConditionWithExpression e) { r("visitWhenConditionWithExpression", e); super.visitWhenConditionWithExpression(e); }
    @Override public void visitObjectDeclaration(KtObjectDeclaration e) { r("visitObjectDeclaration", e); super.visitObjectDeclaration(e); }
    @Override public void visitStringTemplateEntry(KtStringTemplateEntry e) { r("visitStringTemplateEntry", e); super.visitStringTemplateEntry(e); }
    @Override public void visitStringTemplateEntryWithExpression(KtStringTemplateEntryWithExpression e) { r("visitStringTemplateEntryWithExpression", e); super.visitStringTemplateEntryWithExpression(e); }
    @Override public void visitBlockStringTemplateEntry(KtBlockStringTemplateEntry e) { r("visitBlockStringTemplateEntry", e); super.visitBlockStringTemplateEntry(e); }
    @Override public void visitSimpleNameStringTemplateEntry(KtSimpleNameStringTemplateEntry e) { r("visitSimpleNameStringTemplateEntry", e); super.visitSimpleNameStringTemplateEntry(e); }
    @Override public void visitLiteralStringTemplateEntry(KtLiteralStringTemplateEntry e) { r("visitLiteralStringTemplateEntry", e); super.visitLiteralStringTemplateEntry(e); }
    @Override public void visitEscapeStringTemplateEntry(KtEscapeStringTemplateEntry e) { r("visitEscapeStringTemplateEntry", e); super.visitEscapeStringTemplateEntry(e); }
    @Override public void visitPackageDirective(KtPackageDirective e) { r("visitPackageDirective", e); super.visitPackageDirective(e); }

    // No void overloads exist upstream for these; ktfmt overrides the (element, data) form.
    @Override public Void visitFileAnnotationList(KtFileAnnotationList e, Void d) { r("visitFileAnnotationList", e); return super.visitFileAnnotationList(e, d); }
    @Override public Void visitAnnotationUseSiteTarget(KtAnnotationUseSiteTarget e, Void d) { r("visitAnnotationUseSiteTarget", e); return super.visitAnnotationUseSiteTarget(e, d); }
    @Override public Void visitStringInterpolationPrefix(KtStringInterpolationPrefix e, Void d) { r("visitStringInterpolationPrefix", e); return super.visitStringInterpolationPrefix(e, d); }
}
