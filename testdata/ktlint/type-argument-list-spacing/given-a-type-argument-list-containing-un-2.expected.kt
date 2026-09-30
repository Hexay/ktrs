class B<T> : A<T>() {
    override fun x() = super<A>.x()
}