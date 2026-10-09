"A stand-in for rules_kotlin: the lint aspect only reads a rule's kind, tags and srcs."

def _kt_jvm_library_impl(ctx):
    return [DefaultInfo(files = depset(ctx.files.srcs))]

# The name is the contract: lint_ktlint_aspect visits rules whose kind is `kt_jvm_library`.
kt_jvm_library = rule(
    implementation = _kt_jvm_library_impl,
    attrs = {"srcs": attr.label_list(allow_files = [".kt", ".kts"])},
)
