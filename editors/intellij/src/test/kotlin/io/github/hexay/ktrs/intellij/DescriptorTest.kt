package io.github.hexay.ktrs.intellij

import org.junit.Assert.assertEquals
import org.junit.Test
import org.w3c.dom.Element
import javax.xml.parsers.DocumentBuilderFactory

class DescriptorTest {
    private fun descriptor(name: String): Element {
        val stream = requireNotNull(javaClass.classLoader.getResourceAsStream("META-INF/$name")) { name }
        return stream.use { DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(it).documentElement }
    }

    private fun Element.elements(): List<Element> =
        (0 until childNodes.length).map { childNodes.item(it) }.filterIsInstance<Element>().flatMap { listOf(it) + it.elements() }

    @Test
    fun `plugin id and the two optional LSP descriptors`() {
        val plugin = descriptor("plugin.xml")
        assertEquals(KtrsBinary.PLUGIN_ID, plugin.getElementsByTagName("id").item(0).textContent)
        val optional = plugin.elements().filter { it.tagName == "depends" && it.getAttribute("optional") == "true" }
            .associate { it.textContent.trim() to it.getAttribute("config-file") }
        assertEquals(mapOf("com.intellij.modules.lsp" to "ktrs-lsp.xml", "com.redhat.devtools.lsp4ij" to "ktrs-lsp4ij.xml"), optional)
    }

    @Test
    fun `every class a descriptor names exists`() {
        val classAttributes = setOf("implementation", "serviceImplementation", "instance", "class", "factoryClass", "implementationClass", "topic")
        for (name in listOf("plugin.xml", "ktrs-lsp.xml", "ktrs-lsp4ij.xml")) {
            val names = descriptor(name).elements().flatMap { e -> classAttributes.mapNotNull { e.getAttribute(it).ifEmpty { null } } }
            assert(names.isNotEmpty()) { name }
            for (className in names) {
                Class.forName(className, false, javaClass.classLoader)
            }
        }
    }
}
