(cd binding && wasm-pack build) || exit 1
(cd www && npm run start) || exit 1