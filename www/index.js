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

document.getElementById("update").onclick = (function () {
    context = wasm.parse(document.getElementById("input").value);
    document.getElementById("output").innerHTML = context.render();
})

document.getElementById("input").addEventListener('keydown', function (e) {
    if (e.code == "Enter" && (e.ctrlKey || e.metaKey)) {
        document.getElementById("update").click()
    }
});