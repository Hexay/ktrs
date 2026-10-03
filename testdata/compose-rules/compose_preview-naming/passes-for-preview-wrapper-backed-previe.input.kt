@Preview
@PreviewWrapper(CustomThemeWrapper::class)
@Composable
fun SomeFeatureListEmpty() { }

@Preview
@PreviewWrapperProvider(CustomThemeWrapper::class)
@Composable
fun SomeFeatureListLoaded() { }