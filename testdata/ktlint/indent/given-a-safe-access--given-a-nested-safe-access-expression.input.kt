val foo = bar
    ?.filter { number ->
        number == 0
    }?.map { evenNumber ->
        evenNumber * evenNumber
    }?.map { evenNumber ->
        evenNumber * evenNumber
    }?.map { evenNumber ->
        evenNumber * evenNumber
    }