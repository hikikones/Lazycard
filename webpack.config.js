const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const BUILD_DIR = path.resolve(__dirname, "build");
const SRC_DIR = path.resolve(__dirname, "src");

const tsRule = {
    test: /\.ts(x?)$/,
    exclude: /node_modules/,
    use: 'ts-loader',
    resolve: {
        extensions: [".ts", ".tsx"]
    }
};

const cssRule = {
    test: /\.css$/,
    use: ["style-loader", "css-loader"]
};

const fileRule = {
    test: /\.(png|jpe?g|gif|svg|ttf|woff(2)?|eot)$/,
    use: "file-loader"
};

const nodeSettings = {
    __dirname: false,
    __filename: false
};

const mainConfig = {
    target: 'electron-main',
    entry: path.resolve(SRC_DIR, "main", "main.ts"),
    output: {
        path: BUILD_DIR,
        filename: "main.js"
    },
    module: {
        rules: [tsRule]
    },
    node: nodeSettings,
};

const rendererConfig = {
    target: 'electron-renderer',
    entry: path.resolve(SRC_DIR, "renderer", "renderer.tsx"),
    output: {
        path: BUILD_DIR,
        filename: "renderer.js"
    },
    module: {
        rules: [tsRule, cssRule, fileRule]
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: path.resolve(SRC_DIR, "renderer", "index.html"),
            filename: "index.html"
        }),
        new CopyWebpackPlugin({
            patterns: [
              { from: './static', to: BUILD_DIR },
            ],
          }),
    ],
    node: nodeSettings,
};

module.exports = [mainConfig, rendererConfig];