fun foo() { //
    /**.*/
    generateSequence(locate(dir)) { seed -> locate(seed.parent.parent) } // seed.parent == .editorconfig dir
        .map { it to lazy { load(it) } }
}