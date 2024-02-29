#!/bin/sh -xe
git diff --quiet || (git status && false)
(cd binding && wasm-pack build)
(cd www && rm -rf dist && npm run build)
git checkout gh-pages
rm *.wasm
cp www/dist/* .
git add *.wasm
git commit -am update
git push 
git co main
