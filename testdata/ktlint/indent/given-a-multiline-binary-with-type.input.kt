fun foo() {
    node.prevLeaf { it is PsiWhiteSpace && it.textContains('\n') } as
        PsiWhiteSpace?
}