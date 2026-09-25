import org.jetbrains.kotlin.KtNodeTypes;
import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.com.intellij.psi.PsiErrorElement;
import org.jetbrains.kotlin.com.intellij.psi.PsiNameIdentifierOwner;
import org.jetbrains.kotlin.name.FqName;
import org.jetbrains.kotlin.psi.*;
import org.jetbrains.kotlin.resolve.ImportPath;

/** Accessors of declarations, types, files and directives that ktfmt calls. */
final class Decls {
    private Decls() {}

    static void describe(PsiElement e, StringBuilder sb) {
        if (e instanceof KtFile) {
            KtFile x = (KtFile) e;
            Base.line(sb, "KtFile.importList", () -> Fmt.ref(x.getImportList()));
        }
        if (e instanceof PsiErrorElement) {
            PsiErrorElement x = (PsiErrorElement) e;
            Base.line(sb, "PsiErrorElement.errorDescription", () -> Fmt.str(x.getErrorDescription()));
        }
        if (e instanceof KtModifierListOwner) {
            KtModifierListOwner x = (KtModifierListOwner) e;
            Base.line(sb, "KtModifierListOwner.modifierList", () -> Fmt.ref(x.getModifierList()));
        }
        if (e instanceof PsiNameIdentifierOwner) {
            PsiNameIdentifierOwner x = (PsiNameIdentifierOwner) e;
            Base.line(sb, "PsiNameIdentifierOwner.nameIdentifier", () -> Fmt.ref(x.getNameIdentifier()));
        }
        if (e instanceof KtNamedFunction || e instanceof KtClassOrObject || e instanceof KtSecondaryConstructor) {
            KtElementImplStub<?> x = (KtElementImplStub<?>) e;
            Base.line(sb, "getStubOrPsiChild(CONTEXT_PARAMETER_LIST)", () -> Fmt.ref(x.getStubOrPsiChild(KtNodeTypes.CONTEXT_PARAMETER_LIST)));
        }
        if (e instanceof KtTypeParameterListOwner) {
            KtTypeParameterListOwner x = (KtTypeParameterListOwner) e;
            Base.line(sb, "KtTypeParameterListOwner.typeParameterList", () -> Fmt.ref(x.getTypeParameterList()));
            Base.line(sb, "KtTypeParameterListOwner.typeConstraintList", () -> Fmt.ref(x.getTypeConstraintList()));
        }
        if (e instanceof KtCallableDeclaration) {
            KtCallableDeclaration x = (KtCallableDeclaration) e;
            Base.line(sb, "KtCallableDeclaration.receiverTypeReference", () -> Fmt.ref(x.getReceiverTypeReference()));
            Base.line(sb, "KtCallableDeclaration.typeReference", () -> Fmt.ref(x.getTypeReference()));
            Base.line(sb, "KtCallableDeclaration.valueParameterList", () -> Fmt.ref(x.getValueParameterList()));
        }
        if (e instanceof KtDeclarationWithBody) {
            KtDeclarationWithBody x = (KtDeclarationWithBody) e;
            Base.line(sb, "KtDeclarationWithBody.bodyExpression", () -> Fmt.ref(x.getBodyExpression()));
            Base.line(sb, "KtDeclarationWithBody.bodyBlockExpression", () -> Fmt.ref(x.getBodyBlockExpression()));
            Base.line(sb, "KtDeclarationWithBody.valueParameters", () -> Fmt.refs(x.getValueParameters()));
        }
        if (e instanceof KtProperty) {
            KtProperty x = (KtProperty) e;
            Base.line(sb, "KtProperty.valOrVarKeyword", () -> Fmt.ref(x.getValOrVarKeyword()));
            Base.line(sb, "KtProperty.delegate", () -> Fmt.ref(x.getDelegate()));
            Base.line(sb, "KtProperty.initializer", () -> Fmt.ref(x.getInitializer()));
            Base.line(sb, "KtProperty.accessors", () -> Fmt.refs(x.getAccessors()));
            Base.line(sb, "KtProperty.fieldDeclaration", () -> Fmt.ref(x.getFieldDeclaration()));
            Base.line(sb, "KtProperty.getter", () -> Fmt.ref(x.getGetter()));
            Base.line(sb, "KtProperty.setter", () -> Fmt.ref(x.getSetter()));
        }
        if (e instanceof KtPropertyAccessor) {
            KtPropertyAccessor x = (KtPropertyAccessor) e;
            Base.line(sb, "KtPropertyAccessor.namePlaceholder", () -> Fmt.ref(x.getNamePlaceholder()));
            Base.line(sb, "KtPropertyAccessor.returnTypeReference", () -> Fmt.ref(x.getReturnTypeReference()));
            Base.line(sb, "KtPropertyAccessor.parameterList", () -> Fmt.ref(x.getParameterList()));
        }
        if (e instanceof KtBackingField) {
            KtBackingField x = (KtBackingField) e;
            Base.line(sb, "KtBackingField.namePlaceholder", () -> Fmt.ref(x.getNamePlaceholder()));
            Base.line(sb, "KtBackingField.returnTypeReference", () -> Fmt.ref(x.getReturnTypeReference()));
            Base.line(sb, "KtBackingField.initializer", () -> Fmt.ref(x.getInitializer()));
        }
        if (e instanceof KtParameter) {
            KtParameter x = (KtParameter) e;
            Base.line(sb, "KtParameter.destructuringDeclaration", () -> Fmt.ref(x.getDestructuringDeclaration()));
            Base.line(sb, "KtParameter.valOrVarKeyword", () -> Fmt.ref(x.getValOrVarKeyword()));
            Base.line(sb, "KtParameter.defaultValue", () -> Fmt.ref(x.getDefaultValue()));
        }
        if (e instanceof KtParameterList) {
            KtParameterList x = (KtParameterList) e;
            Base.line(sb, "KtParameterList.parameters", () -> Fmt.refs(x.getParameters()));
            Base.line(sb, "KtParameterList.trailingComma", () -> Fmt.ref(x.getTrailingComma()));
            Base.line(sb, "KtParameterList.leftParenthesis", () -> Fmt.ref(x.getLeftParenthesis()));
            Base.line(sb, "KtParameterList.rightParenthesis", () -> Fmt.ref(x.getRightParenthesis()));
        }
        if (e instanceof KtTypeReference) {
            KtTypeReference x = (KtTypeReference) e;
            Base.line(sb, "KtTypeReference.typeElement", () -> Fmt.ref(x.getTypeElement()));
        }
        if (e instanceof KtNullableType) {
            KtNullableType x = (KtNullableType) e;
            Base.line(sb, "KtNullableType.modifierList", () -> Fmt.ref(x.getModifierList()));
            Base.line(sb, "KtNullableType.innerType", () -> Fmt.ref(x.getInnerType()));
        }
        if (e instanceof KtUserType) {
            KtUserType x = (KtUserType) e;
            Base.line(sb, "KtUserType.qualifier", () -> Fmt.ref(x.getQualifier()));
            Base.line(sb, "KtUserType.referenceExpression", () -> Fmt.ref(x.getReferenceExpression()));
            Base.line(sb, "KtUserType.typeArgumentList", () -> Fmt.ref(x.getTypeArgumentList()));
        }
        if (e instanceof KtIntersectionType) {
            KtIntersectionType x = (KtIntersectionType) e;
            Base.line(sb, "KtIntersectionType.leftTypeRef", () -> Fmt.ref(x.getLeftTypeRef()));
            Base.line(sb, "KtIntersectionType.rightTypeRef", () -> Fmt.ref(x.getRightTypeRef()));
        }
        if (e instanceof KtTypeArgumentList) {
            KtTypeArgumentList x = (KtTypeArgumentList) e;
            Base.line(sb, "KtTypeArgumentList.arguments", () -> Fmt.refs(x.getArguments()));
            Base.line(sb, "KtTypeArgumentList.trailingComma", () -> Fmt.ref(x.getTrailingComma()));
        }
        if (e instanceof KtTypeProjection) {
            KtTypeProjection x = (KtTypeProjection) e;
            Base.line(sb, "KtTypeProjection.typeReference", () -> Fmt.ref(x.getTypeReference()));
            Base.line(sb, "KtTypeProjection.projectionKind", () -> x.getProjectionKind().name());
        }
        if (e instanceof KtFunctionType) {
            KtFunctionType x = (KtFunctionType) e;
            Base.line(sb, "KtFunctionType.contextReceiverList", () -> Fmt.ref(x.getContextReceiverList()));
            Base.line(sb, "KtFunctionType.receiver", () -> Fmt.ref(x.getReceiver()));
            Base.line(sb, "KtFunctionType.parameterList", () -> Fmt.ref(x.getParameterList()));
            Base.line(sb, "KtFunctionType.returnTypeReference", () -> Fmt.ref(x.getReturnTypeReference()));
        }
        Classes.describe(e, sb);
        if (e instanceof KtPackageDirective) {
            KtPackageDirective x = (KtPackageDirective) e;
            Base.line(sb, "KtPackageDirective.packageKeyword", () -> Fmt.ref(x.getPackageKeyword()));
            Base.line(sb, "KtPackageDirective.packageNames", () -> Fmt.refs(x.getPackageNames()));
            Base.line(sb, "KtPackageDirective.fqName", () -> Fmt.str(x.getFqName().asString()));
        }
        if (e instanceof KtImportList) {
            KtImportList x = (KtImportList) e;
            Base.line(sb, "KtImportList.imports", () -> Fmt.refs(x.getImports()));
        }
        if (e instanceof KtImportDirective) {
            KtImportDirective x = (KtImportDirective) e;
            Base.line(sb, "KtImportDirective.importedReference", () -> Fmt.ref(x.getImportedReference()));
            Base.line(sb, "KtImportDirective.isAllUnder", () -> String.valueOf(x.isAllUnder()));
            Base.line(sb, "KtImportDirective.alias", () -> Fmt.ref(x.getAlias()));
            Base.line(sb, "KtImportDirective.importedFqName", () -> fqName(x.getImportedFqName()));
            Base.line(sb, "KtImportDirective.isValidImport", () -> String.valueOf(x.isValidImport()));
            Base.line(sb, "KtImportDirective.importPath.importedName", () -> {
                ImportPath path = x.getImportPath();
                if (path == null) return "null";
                return path.getImportedName() == null ? "null" : Fmt.str(path.getImportedName().asString());
            });
        }
        if (e instanceof KtScript) {
            KtScript x = (KtScript) e;
            Base.line(sb, "KtScript.blockExpression", () -> Fmt.ref(x.getBlockExpression()));
        }
    }

    /** asString / shortName / parent: ktfmt compares parent() with the package and reads shortName(). */
    private static String fqName(FqName fq) {
        if (fq == null) return "null";
        String shortName = Fmt.safe(() -> Fmt.str(fq.shortName().asString()));
        String parent = Fmt.safe(() -> Fmt.str(fq.parent().asString()));
        return Fmt.str(fq.asString()) + " short=" + shortName + " parent=" + parent;
    }
}
