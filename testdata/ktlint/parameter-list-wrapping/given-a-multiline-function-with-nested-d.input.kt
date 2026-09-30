fun visit(
    node: ASTNode,
        autoCorrect: Boolean,
    emit: (offset: Int, errorMessage: String,
    canBeAutoCorrected: Boolean) -> Unit) {}