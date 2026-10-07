package demo
import kotlin.math.max
import kotlin.collections.List
class Messy( val a:Int,val b : String ,private val c:List<String> ){
fun  compute(x:Int,y:Int):Int{ val z=max(x,y)
    if(z>10){return z*2} else {
            return z }
  }
    fun longCall() = listOf("alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta").map { it.uppercase() }.filter { it.length > 3 }
}
