import * as wasm from "../binding/pkg/binding.js";

document.getElementById("browse").addEventListener("change", handleFiles, false);

var context = undefined;
const update_button = document.getElementById("update");
const input = document.getElementById('input');
const output = document.getElementById("output");
const generate_button = document.getElementById("generate");

function handleFiles() {
    const reader = new FileReader();
    reader.onload = (function (x) {
        input.value = x.target.result;
        update_button.click();
    });
    reader.readAsText(this.files[0]);
}

update_button.onclick = function () {
    context = wasm.parse(input.value);
    const is_error = context.is_error();
    output.innerHTML = context.render();
    if (!is_error) {
        svgPanZoom(output.childNodes[0]);
    }
}

generate_button.onclick = function () {
    const nodes_count = document.getElementById("nodes_count");
    nodes_count.value = Math.min(nodes_count.value, nodes_count.max);
    const edges_count = document.getElementById("edges_count");
    edges_count.value = Math.min(edges_count.value, edges_count.max);
    input.value = wasm.render_random(nodes_count.value, edges_count.value);
    document.getElementById('update').click();
}

var lastId = undefined;
var lastColor;
function outputClickHandler(id) {
    if (lastId) {
        document.getElementById(lastId).setAttribute('fill', lastColor);
    }
    const item = document.getElementById(id);
    lastColor = item.getAttribute('fill')
    lastId = id;
    item.setAttribute('fill', 'green')
}

function visualize(data) {
    input.value = 'digraph g {' + data + '}'
    update_button.click()
}

// export functions
document.visualize = visualize;
document.outputClickHandler = outputClickHandler;

// create graph we have input (nice for page reload)
if (input.value.length != 0) {
    update_button.click();
}