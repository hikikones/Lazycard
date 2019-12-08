const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');

const BUILD_DIR = path.resolve(__dirname, "build");
const APP_DIR = path.resolve(__dirname, "src");

const tsRule = {
    test: /\.ts(x?)$/,
    exclude: /node_modules/,
    use: 'ts-loader',
    resolve: {
        extensions: [".ts", ".tsx"]
    }
};

const jsRule = {
    test: /\.js$/,
    use: 'source-map-loader',
    enforce: 'pre'
};

const mainConfig = {
    target: 'electron-main',
    entry: path.resolve(APP_DIR, "main", "main.ts"),
    devtool: 'source-map',
    output: {
        path: BUILD_DIR,
        filename: "main.js"
    },
    module: {
        rules: [tsRule, jsRule]
    }
};

const rendererConfig = {
    target: 'electron-renderer',
    entry: path.resolve(APP_DIR, "renderer", "renderer.tsx"),
    devtool: 'source-map',
    output: {
        path: BUILD_DIR,
        filename: "renderer.js"
    },
    module: {
        rules: [tsRule, jsRule]
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: path.resolve(APP_DIR, "renderer", "index.html"),
            filename: "index.html"
        })
    ]
};

module.exports = [mainConfig, rendererConfig];