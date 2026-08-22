
# Need some sort of global build

#S plit into sections, and have an option to do all

# 1. building vanguard
# 2. bundling the client
# 3. building the server

# mkdir -p public/deps/blight

# cp node_modules/@mercuryworkshop/scramjet/dist/scramjet.js public/deps/blight/
# cp node_modules/@mercuryworkshop/scramjet/dist/scramjet.wasm public/deps/blight/
# cp node_modules/@mercuryworkshop/scramjet-controller/dist/controller.inject.js public/deps/blight/
# cp node_modules/@mercuryworkshop/scramjet-controller/dist/controller.api.js public/deps/blight/
# cp node_modules/@mercuryworkshop/scramjet-controller/dist/controller.sw.js public/deps/blight/


# mkdir -p public/deps/blight/vanguard

# cp ../packages/vanguard/vanguard_bg.wasm public/deps/blight/

# npx esbuild src/blight/sw.ts --bundle --outfile=public/sw.js --format=esm --target=es2022


# https://github.com/gorhill/uBlock/tree/master/src/web_accessible_resources
# https://github.com/gorhill/uBlock/blob/master/src/js/redirect-resources.js
# https://github.com/gorhill/uBlock/blob/master/src/js/resources/scriptlets.js

# server is independent. Building client depends on building vanguard. So check if js exists


build_server() {
    cargo b $1
}

build_vanguard() {
    cd packages/vanguard
    node build.js $1
    cd ../..
}

# $1 = opt lvl, $2 = target
build_client() {
    if [ $2 -eq "vite" ]; then
    
}