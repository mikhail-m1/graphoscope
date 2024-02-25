#!/bin/sh -xe
(cd binding && wasm-pack build)
(cd www && npm run build)
git checkout gh-pages
rm *.wasm
cp www/dist/* .
git add *.wasm
git commit -am update
echo git push 
echo git co main
