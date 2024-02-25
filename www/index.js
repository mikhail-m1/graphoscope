import * as wasm from "../binding/pkg/binding.js";

document.getElementById("browse").addEventListener("change", handleFiles, false);

var context = undefined;
const update_button = document.getElementById("update");
const input = document.getElementById('input');
const output = document.getElementById("output");
const generate_button = document.getElementById("generate");
const max_nodes = document.getElementById("max_nodes");
const max_edges = document.getElementById("max_edges");
const nodes_count = document.getElementById("nodes_count");
const edges_count = document.getElementById("edges_count");
var currentId = "";
var lastColor;

function handleFiles() {
    const reader = new FileReader();
    reader.onload = (function (x) {
        input.value = x.target.result;
        update_button.click();
    });
    reader.readAsText(this.files[0]);
}

function update_render() {
    max_nodes.value = Math.min(max_nodes.value, max_nodes.max);
    max_edges.value = Math.min(max_edges.value, max_edges.max);
    output.innerHTML = context.render(currentId, max_nodes.value, max_edges.value);
    if (!context.is_error()) {
        svgPanZoom(output.childNodes[0]);
    }
}

update_button.onclick = function () {
    context = wasm.parse(input.value);
    update_render();
}

generate_button.onclick = function () {
    nodes_count.value = Math.min(nodes_count.value, nodes_count.max);
    edges_count.value = Math.min(edges_count.value, edges_count.max);
    input.value = wasm.render_random(nodes_count.value, edges_count.value);
    document.getElementById('update').click();
}

document.outputClickHandler = id => {
    if (currentId) {
        document.getElementById(currentId).setAttribute('fill', lastColor);
    }
    currentId = id;
    update_render();
    const item = document.getElementById(id);
    lastColor = item.getAttribute('fill')
    currentId = id;
    item.setAttribute('fill', 'green')
}

document.visualize = data => {
    input.value = 'digraph g {' + data + '}'
    update_button.click()
}

// create graph we have input (nice for page reload)
if (input.value.length != 0) {
    update_button.click();
}