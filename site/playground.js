// The playground: ktrs.wasm (crates/ktrs-wasm) through its plain exports; state lives in the URL hash.

const SAMPLE = `package com.example.playground

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.util.Locale // unused: ktfmt removes it

data class User(val id: Long, val name: String, val email: String?, val roles: List<String> = emptyList())

class UserRepository(private val api: Api, private val cache: MutableMap<Long, User> = mutableMapOf()) {
    fun observe(id: Long): Flow<User> = api.userUpdates(id).map { dto -> User(dto.id, dto.name.trim(), dto.email?.lowercase(), dto.roles.filter { it.isNotBlank() }) }

    suspend fun load(id: Long): User? {
        val cached = cache[id]; if (cached != null) return cached
        return api.fetch(id)?.also { cache[id] = it }
    }
}
`;

const $ = (id) => document.getElementById(id);
const input = $("input"), output = $("output"), error = $("error"), status = $("status");
const style = $("style"), width = $("width");
let wasm;

async function load() {
  const response = fetch("ktrs.wasm");
  const { instance } = WebAssembly.instantiateStreaming
    ? await WebAssembly.instantiateStreaming(response)
    : await WebAssembly.instantiate(await (await response).arrayBuffer());
  wasm = instance.exports;
}

/** [ok, text]: the formatted code, or ktfmt's error message. */
function format(code) {
  const bytes = new TextEncoder().encode(code);
  const ptr = wasm.alloc(bytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  const out = wasm.format(ptr, bytes.length, Number(style.value), Number(width.value) || 0);
  const view = new DataView(wasm.memory.buffer);
  const ok = view.getUint8(out) === 0, len = view.getUint32(out + 1, true);
  const text = new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, out + 5, len));
  wasm.dealloc(out, 5 + len);
  return [ok, text];
}

function show(message, kind) {
  status.textContent = message;
  status.className = kind || "";
}

async function run() {
  if (!wasm) return;
  const code = input.value;
  const start = performance.now();
  let ok, text;
  try {
    [ok, text] = format(code);
  } catch (e) {
    // A trap (a formatter bug) leaves the instance unusable: start a fresh one.
    await load();
    show("Internal error in ktrs; please report this input", "error");
    return;
  }
  const ms = performance.now() - start;
  if (ok) {
    output.value = text;
    error.style.display = "none";
    show(text === code ? `Already formatted (${ms.toFixed(1)} ms)` : `Formatted in ${ms.toFixed(1)} ms`, "ok");
  } else {
    error.textContent = text;
    error.style.display = "block";
    show("Can't format: the input doesn't parse", "error");
  }
  saveState();
}

function encode(text) {
  let binary = "";
  for (const byte of new TextEncoder().encode(text)) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function decode(text) {
  const binary = atob(text.replace(/-/g, "+").replace(/_/g, "/"));
  return new TextDecoder().decode(Uint8Array.from(binary, (c) => c.charCodeAt(0)));
}

function saveState() {
  const params = new URLSearchParams({ style: style.value, code: encode(input.value) });
  if (width.value) params.set("width", width.value);
  history.replaceState(null, "", "#" + params);
}

function restoreState() {
  const params = new URLSearchParams(location.hash.slice(1));
  try {
    input.value = params.has("code") ? decode(params.get("code")) : SAMPLE;
  } catch {
    input.value = SAMPLE;
  }
  if (["0", "1", "2"].includes(params.get("style"))) style.value = params.get("style");
  if (params.get("width")) width.value = params.get("width");
}

let pending;
function schedule() {
  clearTimeout(pending);
  pending = setTimeout(run, 120);
}

async function copy(text, button) {
  await navigator.clipboard.writeText(text);
  const label = button.textContent;
  button.textContent = "Copied";
  setTimeout(() => (button.textContent = label), 1200);
}

restoreState();
input.addEventListener("input", schedule);
style.addEventListener("change", run);
width.addEventListener("input", schedule);
$("copy").addEventListener("click", (e) => copy(output.value, e.target));
$("share").addEventListener("click", (e) => { saveState(); copy(location.href, e.target); });
input.addEventListener("keydown", (e) => {
  if (e.key === "Tab" && !e.shiftKey) {
    e.preventDefault();
    input.setRangeText("    ", input.selectionStart, input.selectionEnd, "end");
    schedule();
  }
});

load().then(run, (e) => show("Couldn't load the formatter: " + e.message, "error"));
