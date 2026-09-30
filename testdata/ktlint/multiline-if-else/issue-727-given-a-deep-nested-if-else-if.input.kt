fun main() {
    if (outerCondition1)
        if (innerCondition1)
            if (innerCondition11)
                return 0
            else if (innerCondition12)
                return 12
            else if (innerCondition13)
                return 13
            else
                return 14
        else if (innerCondition44)
            return 1
        else {
            return 16
        }
    else if (outerCondition2)
        if (innerCondition2)
            return 2
        else if (innerCondition3)
            return 3
        else
            return 4
    else
        if (innerCondition4)
            return 5
        else
            return -1
}