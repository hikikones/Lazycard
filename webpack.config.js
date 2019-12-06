const HtmlWebpackPlugin = require('html-webpack-plugin');

const BUILD_DIR = __dirname + "/build";
const APP_DIR = __dirname + "/src";

const tsRule = {
    test: /\.ts(x?)$/,
    use: 'ts-loader'
};

const mainConfig = {
    target: 'electron-main',
    entry: APP_DIR + "/main/main.ts",
    output: {
        path: BUILD_DIR,
        filename: "main.js"
    },
    module: {
        rules: [tsRule]
    }
};

const rendererConfig = {
    target: 'electron-renderer',
    entry: APP_DIR + "/renderer/index.tsx",
    output: {
        path: BUILD_DIR,
        filename: "index.js"
    },
    module: {
        rules: [tsRule]
    },
    plugins: [
        new HtmlWebpackPlugin({template: APP_DIR + "/renderer/index.html"})
    ]
};

module.exports = [mainConfig, rendererConfig];