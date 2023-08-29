const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

const sysbadge = process.env.SYSBADGE_WASM_PATH ?
    path.resolve(process.env.SYSBADGE_WASM_PATH, "bundler") :
    path.resolve(__dirname, "../target/wasm32-unknown-unknown/release/pkg/");

console.log("Using sysbadge wasm from: " + sysbadge)

module.exports = {
    mode: "production",
    entry: {
        index: "./js/index.js"
    },
    resolve: {
        alias: {
            Sysbadge: sysbadge
        }
    },
    output: {
        path: dist,
        filename: "[name].js"
    },
    devServer: {
        contentBase: dist,
    },
    plugins: [
        new CopyPlugin([
            path.resolve(__dirname, "static")
        ]),
    ],
    experiments: {
        asyncWebAssembly: true
    },
};