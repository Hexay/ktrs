class BiAdapter<C : RecyclerView.ViewHolder, V1 : C, V2 : C, out A1, out A2>(
    val adapter1: A1,
    val adapter2: A2
) : RecyclerView.Adapter<C>()
    where A1 : RecyclerView.Adapter<V1>, A1 : ComposableAdapter.ViewTypeProvider,
          A2 : RecyclerView.Adapter<V2>, A2 : ComposableAdapter.ViewTypeProvider
     
		