val foo =
    if (true)
        try { 1 } catch (e: Exception) { 2 }
    else 3