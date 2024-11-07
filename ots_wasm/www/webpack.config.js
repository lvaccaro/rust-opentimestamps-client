const CopyWebpackPlugin = require("copy-webpack-plugin");
//const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({ patterns: ["index.html"] })
  ],
  experiments: {
    asyncWebAssembly: true,
    syncWebAssembly: true
  }
};