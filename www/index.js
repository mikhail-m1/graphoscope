import * as wasm from "../binding/pkg/binding.js";

const inputElement = document.getElementById("browse");
inputElement.addEventListener("change", handleFiles, false);

var context = undefined;

function handleFiles() {
    //console.log(this.files); /* now you can work with the file list */
    const reader = new FileReader();
    reader.onload = (function (x) {
        // console.log(x.target.result);
        document.getElementById("input").value = x.target.result;
        context = wasm.parse(x.target.result);
        // console.log(context.node_count())
        document.getElementById("output").innerHTML = context.render();
    });
    reader.readAsText(this.files[0]);
}

document.getElementById("update").onclick = function () {
    context = wasm.parse(document.getElementById("input").value);
    document.getElementById("output").innerHTML = context.render();
    svgPanZoom(output.childNodes[0]);
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
    document.getElementById("input").value = 'digraph g {' + data + '}'
    document.getElementById("update").click()
}

// export functions
document.visualize = visualize;
document.outputClickHandler = outputClickHandler;

// create graph we have input (nice for page reload)
if (document.getElementById('input').value.length != 0) {
    document.getElementById('update').click();
}