import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.psi.*;

/** Accessors of classes, constructors, super types, type parameters, annotations and context lists. */
final class Classes {
    private Classes() {}

    static void describe(PsiElement e, StringBuilder sb) {
        if (e instanceof KtClassOrObject) {
            KtClassOrObject x = (KtClassOrObject) e;
            Base.line(sb, "KtClassOrObject.declarationKeyword", () -> Fmt.ref(x.getDeclarationKeyword()));
            Base.line(sb, "KtClassOrObject.primaryConstructor", () -> Fmt.ref(x.getPrimaryConstructor()));
            Base.line(sb, "KtClassOrObject.superTypeList", () -> Fmt.ref(x.getSuperTypeList()));
            Base.line(sb, "KtClassOrObject.body", () -> Fmt.ref(x.getBody()));
        }
        if (e instanceof KtClass) {
            KtClass x = (KtClass) e;
            Base.line(sb, "KtClass.isEnum", () -> String.valueOf(x.isEnum()));
        }
        if (e instanceof KtObjectDeclaration) {
            KtObjectDeclaration x = (KtObjectDeclaration) e;
            Base.line(sb, "KtObjectDeclaration.isCompanion", () -> String.valueOf(x.isCompanion()));
        }
        if (e instanceof KtEnumEntry) {
            KtEnumEntry x = (KtEnumEntry) e;
            Base.line(sb, "KtEnumEntry.initializerList", () -> Fmt.ref(x.getInitializerList()));
        }
        if (e instanceof KtInitializerList) {
            KtInitializerList x = (KtInitializerList) e;
            Base.line(sb, "KtInitializerList.initializers", () -> Fmt.refs(x.getInitializers()));
        }
        if (e instanceof KtSuperTypeList) {
            KtSuperTypeList x = (KtSuperTypeList) e;
            Base.line(sb, "KtSuperTypeList.entries", () -> Fmt.refs(x.getEntries()));
        }
        if (e instanceof KtDelegatedSuperTypeEntry) {
            KtDelegatedSuperTypeEntry x = (KtDelegatedSuperTypeEntry) e;
            Base.line(sb, "KtDelegatedSuperTypeEntry.typeReference", () -> Fmt.ref(x.getTypeReference()));
            Base.line(sb, "KtDelegatedSuperTypeEntry.delegateExpression", () -> Fmt.ref(x.getDelegateExpression()));
        }
        if (e instanceof KtCallElement) {
            KtCallElement x = (KtCallElement) e;
            Base.line(sb, "KtCallElement.calleeExpression", () -> Fmt.ref(x.getCalleeExpression()));
            Base.line(sb, "KtCallElement.typeArgumentList", () -> Fmt.ref(x.getTypeArgumentList()));
            Base.line(sb, "KtCallElement.valueArgumentList", () -> Fmt.ref(x.getValueArgumentList()));
            Base.line(sb, "KtCallElement.lambdaArguments", () -> Fmt.refs(x.getLambdaArguments()));
        }
        if (e instanceof KtConstructor) {
            KtConstructor<?> x = (KtConstructor<?>) e;
            Base.line(sb, "KtConstructor.hasConstructorKeyword", () -> String.valueOf(x.hasConstructorKeyword()));
        }
        if (e instanceof KtSecondaryConstructor) {
            KtSecondaryConstructor x = (KtSecondaryConstructor) e;
            Base.line(sb, "KtSecondaryConstructor.delegationCall", () -> Fmt.ref(x.getDelegationCall()));
        }
        if (e instanceof KtConstructorDelegationCall) {
            KtConstructorDelegationCall x = (KtConstructorDelegationCall) e;
            Base.line(sb, "KtConstructorDelegationCall.isImplicit", () -> String.valueOf(x.isImplicit()));
            Base.line(sb, "KtConstructorDelegationCall.isCallToThis", () -> String.valueOf(x.isCallToThis()));
        }
        if (e instanceof KtAnonymousInitializer) {
            KtAnonymousInitializer x = (KtAnonymousInitializer) e;
            Base.line(sb, "KtAnonymousInitializer.body", () -> Fmt.ref(x.getBody()));
        }
        if (e instanceof KtTypeAlias) {
            KtTypeAlias x = (KtTypeAlias) e;
            Base.line(sb, "KtTypeAlias.typeReference", () -> Fmt.ref(x.getTypeReference()));
        }
        if (e instanceof KtTypeParameterList) {
            KtTypeParameterList x = (KtTypeParameterList) e;
            Base.line(sb, "KtTypeParameterList.parameters", () -> Fmt.refs(x.getParameters()));
            Base.line(sb, "KtTypeParameterList.trailingComma", () -> Fmt.ref(x.getTrailingComma()));
        }
        if (e instanceof KtTypeParameter) {
            KtTypeParameter x = (KtTypeParameter) e;
            Base.line(sb, "KtTypeParameter.extendsBound", () -> Fmt.ref(x.getExtendsBound()));
        }
        if (e instanceof KtTypeConstraintList) {
            KtTypeConstraintList x = (KtTypeConstraintList) e;
            Base.line(sb, "KtTypeConstraintList.constraints", () -> Fmt.refs(x.getConstraints()));
        }
        if (e instanceof KtTypeConstraint) {
            KtTypeConstraint x = (KtTypeConstraint) e;
            Base.line(sb, "KtTypeConstraint.subjectTypeParameterName", () -> Fmt.ref(x.getSubjectTypeParameterName()));
            Base.line(sb, "KtTypeConstraint.boundTypeReference", () -> Fmt.ref(x.getBoundTypeReference()));
        }
        if (e instanceof KtDestructuringDeclaration) {
            KtDestructuringDeclaration x = (KtDestructuringDeclaration) e;
            Base.line(sb, "KtDestructuringDeclaration.valOrVarKeyword", () -> Fmt.ref(x.getValOrVarKeyword()));
            Base.line(sb, "KtDestructuringDeclaration.trailingComma", () -> Fmt.ref(x.getTrailingComma()));
            Base.line(sb, "KtDestructuringDeclaration.lPar", () -> Fmt.ref(x.getLPar()));
            Base.line(sb, "KtDestructuringDeclaration.rPar", () -> Fmt.ref(x.getRPar()));
            Base.line(sb, "KtDestructuringDeclaration.entries", () -> Fmt.refs(x.getEntries()));
            Base.line(sb, "KtDestructuringDeclaration.initializer", () -> Fmt.ref(x.getInitializer()));
        }
        if (e instanceof KtDestructuringDeclarationEntry) {
            KtDestructuringDeclarationEntry x = (KtDestructuringDeclarationEntry) e;
            Base.line(sb, "KtDestructuringDeclarationEntry.initializer", () -> Fmt.ref(x.getInitializer()));
        }
        if (e instanceof KtContextParameterList) {
            KtContextParameterList x = (KtContextParameterList) e;
            Base.line(sb, "KtContextParameterList.contextParameters", () -> Fmt.refs(x.contextParameters()));
            Base.line(sb, "KtContextParameterList.contextReceivers", () -> Fmt.refs(x.contextReceivers()));
        }
        if (e instanceof KtAnnotation) {
            KtAnnotation x = (KtAnnotation) e;
            Base.line(sb, "KtAnnotation.useSiteTarget", () -> Fmt.ref(x.getUseSiteTarget()));
            Base.line(sb, "KtAnnotation.entries", () -> Fmt.refs(x.getEntries()));
        }
        if (e instanceof KtAnnotationUseSiteTarget) {
            KtAnnotationUseSiteTarget x = (KtAnnotationUseSiteTarget) e;
            Base.line(sb, "KtAnnotationUseSiteTarget.renderName", () -> Fmt.str(x.getAnnotationUseSiteTarget().getRenderName()));
        }
        if (e instanceof KtAnnotationEntry) {
            KtAnnotationEntry x = (KtAnnotationEntry) e;
            Base.line(sb, "KtAnnotationEntry.atSymbol", () -> Fmt.ref(x.getAtSymbol()));
            Base.line(sb, "KtAnnotationEntry.useSiteTarget", () -> Fmt.ref(x.getUseSiteTarget()));
        }
    }
}
