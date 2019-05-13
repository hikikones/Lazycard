const path = require("path");
const HtmlWebPackPlugin = require("html-webpack-plugin");
const nodeExternals = require("webpack-node-externals");

const BUILD_DIR = path.resolve(__dirname, "build");
const APP_DIR = path.resolve(__dirname, "src");

const jsRule = {
  test: /\.(jsx?)$/,
  exclude: /node_modules/,
  use: {
    loader: "babel-loader",
    options: {
      presets: ["@babel/preset-env", "@babel/preset-react"]
    }
  },
  resolve: {
    extensions: [".js", ".jsx"]
  }
};

const htmlRule = {
  test: /\.html$/,
  use: "html-loader"
};

const cssRule = {
  test: /\.css$/,
  use: ["style-loader", "css-loader"]
};

const htmlPlugin = new HtmlWebPackPlugin({
  template: APP_DIR + "/index.html",
  filename: "index.html"
});

const nodeSettings = {
  __dirname: false,
  __filename: false
};

const externalsList = [nodeExternals()];

const mainConfig = {
  target: "electron-main",
  entry: APP_DIR + "/main.js",
  output: {
    filename: "main.js",
    path: BUILD_DIR
  },
  module: {
    rules: [jsRule]
  },
  node: nodeSettings,
  externals: externalsList,
  devServer: {
    contentBase: BUILD_DIR,
    port: 9000
  }
};

const rendererConfig = {
  target: "electron-renderer",
  entry: APP_DIR + "/renderer.js",
  output: {
    filename: "renderer.js",
    path: BUILD_DIR
  },
  module: {
    rules: [jsRule, htmlRule, cssRule]
  },
  plugins: [htmlPlugin],
  node: nodeSettings,
  externals: externalsList
};

module.exports = [mainConfig, rendererConfig];
