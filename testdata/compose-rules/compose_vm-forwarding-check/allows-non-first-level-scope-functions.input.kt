@Composable
fun MyComposable(viewModel: MyViewModel) {
    viewModel.also {
        Row {
            AnotherComposable(it)
        }
    }
}
@Composable
fun MyComposable2(viewModel: MyViewModel) {
    viewModel.let {
        SomeComposable {
            AnotherComposable(it)
        }
    }
}
@Composable
fun MyComposable3(viewModel: MyViewModel) {
    viewModel.run {
        Row {
            AnotherComposable(this)
        }
    }
}
@Composable
fun MyComposable4(viewModel: MyViewModel) {
    viewModel.apply {
        Row {
            AnotherComposable(this)
        }
    }
}
@Composable
fun MyComposable5(viewModel: MyViewModel) {
    with(viewModel) {
        SomeComposable {
            AnotherComposable(this)
        }
    }
}