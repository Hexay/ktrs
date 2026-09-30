val editorConfigDefaults = EditorConfigDefaults.load(
    editorConfigPath
        ?.expandTildeToFullPath()
        ?.let { path -> Paths.get(path) },
)