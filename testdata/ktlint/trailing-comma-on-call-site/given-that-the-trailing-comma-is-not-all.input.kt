val list1: List<String,> = emptyList()
val list2: List<
    String, // The comma before the comment should be removed without removing the comment itself
> = emptyList()
val list3: List<
    String, /* The comma before the comment should be removed without removing the comment itself */
    > = emptyList()