// Prints the ids of ktlint's standard rules, one per line (cli-diff.sh disables the ones ktrs lacks).
//   java -cp tools/ktlint-oracle/lib/ktlint-cli-2.0.0-ALPHA-4-all.jar tools/ktlint-oracle/RuleIds.java [provider class]
// The provider class defaults to 2.0's; 1.8.0's is com.pinterest.ktlint.ruleset.standard.StandardRuleSetProvider.
import java.lang.reflect.Method;
import java.util.Collection;

public class RuleIds {
    public static void main(String[] args) throws Exception {
        String providerClass = args.length > 0 ? args[0] : "io.github.ktlint.core.ruleset.standard.StandardRuleSetProvider";
        Object ruleSetProvider = Class.forName(providerClass).getDeclaredConstructor().newInstance();
        for (Object provider : (Collection<?>) ruleSetProvider.getClass().getMethod("getRuleProviders").invoke(ruleSetProvider)) {
            Object rule = provider.getClass().getMethod("createNewRuleInstance").invoke(provider);
            // RuleId is a Kotlin value class: its getter's name may be mangled; reflection boxes it.
            for (Method m : rule.getClass().getMethods()) {
                if (m.getName().startsWith("getRuleId") && m.getParameterCount() == 0) {
                    System.out.println(String.valueOf(m.invoke(rule)).replaceAll("^RuleId\\(value=(.*)\\)$", "$1"));
                    break;
                }
            }
        }
    }
}
