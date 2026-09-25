fun build() {
  dependencies { implementation(libs.androidx.activity) }
  val state = remember { mutableStateOf(false) }
}
